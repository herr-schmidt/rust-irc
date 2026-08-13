mod f_to_b_commands;
mod parsing;
mod structs;

use crate::f_to_b_commands::process_user_input_line;
use crate::parsing::{extract_channel_users_vector, parse_message};
use crate::structs::{ChannelsState, HistoryState, Message, User, WriteTLSStreamState};
use std::str::from_utf8;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{split, AsyncReadExt};
use tokio::net::TcpStream;

use tokio_native_tls::{native_tls, TlsConnector, TlsStream};

async fn connect_to_network(network: &str, port: u32) -> TlsStream<TcpStream> {
    let connector: TlsConnector =
        TlsConnector::from(native_tls::TlsConnector::builder().build().unwrap());

    let stream: TcpStream = TcpStream::connect(format!("{network}:{port}"))
        .await
        .unwrap();
    let tls_stream: TlsStream<TcpStream> = connector.connect(network, stream).await.unwrap();

    return tls_stream;
}

async fn start_irc_listener(app_handle: AppHandle) -> std::io::Result<()> {
    let mut message_text_buffer: String = String::from("");
    let tls_stream: TlsStream<TcpStream> = connect_to_network("irc.libera.chat", 6697).await;
    let (mut read_tls_stream, mut write_tls_stream) = split(tls_stream);

    app_handle.manage(WriteTLSStreamState {
        write_tls_stream: Arc::new(tokio::sync::Mutex::new(write_tls_stream)),
    });

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        let bytes_read: usize = read_tls_stream.read(&mut buffer).await?;

        if bytes_read == 0 {
            println!("Server disconnected.");
            break;
        }

        let buffer_to_text = from_utf8(&buffer[..bytes_read]);

        match buffer_to_text {
            Ok(text) => {
                message_text_buffer.push_str(text);
                let mut messages: Vec<String> = message_text_buffer
                    .split("\r\n")
                    .map(|s| s.to_string())
                    .collect();

                // the message buffer does not end with a CRLF, so the last message is incomplete: keep it into the buffer after clearing it
                if !message_text_buffer.ends_with("\r\n") {
                    message_text_buffer.clear();
                    message_text_buffer.push_str(messages.pop().unwrap().as_str());
                } else {
                    message_text_buffer.clear();
                }

                for message in messages {
                    if !message.is_empty() {
                        println!("{}", message);
                        let message_struct: Message = parse_message(&String::from(message).to_string());
                        extract_channel_users_vector(&message_struct);
                        app_handle
                            .emit(
                                "new-users",
                                [User {
                                    nickname: String::from("asd"),
                                    real_name: String::from("dds"),
                                }],
                            )
                            .unwrap();
                    }
                }
            }
            Err(_) => {
                println!("Server disconnected.");
                break;
            }
        };
    }

    // let res = thread_join_handle.join();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.manage(ChannelsState {
                channels_data: Default::default(),
            });
            app.manage(HistoryState {
                history: Default::default(),
            });

            // this spawns a new OS thread
            std::thread::spawn(move || {
                // need a tokio runtime here, since the start_irc_listener is async
                let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(async {
                    if let Err(e) = start_irc_listener(app_handle).await {
                        eprintln!("IRC listener encountered an error: {}", e);
                    }
                });
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![process_user_input_line])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
