use std::str::from_utf8;
use tauri::AppHandle;
use tokio::io::{self, split, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls, TlsConnector, TlsStream};

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    nickname: String,
    real_name: String,
}

#[tauri::command]
fn greet() -> Vec<User> {
    let mut users: Vec<User> = vec![];
    for i in 0..64 {
        users.push(User {
            nickname: String::from(format!("User {} Nickname", i)),
            real_name: String::from(format!("User {} Real Name", i)),
        });
    }
    return users;
}

async fn connect_to_network(network: &str, port: u32) -> TlsStream<TcpStream> {
    let connector = TlsConnector::from(native_tls::TlsConnector::builder().build().unwrap());

    let stream = TcpStream::connect(format!("{network}:{port}"))
        .await
        .unwrap();
    let tls_stream = connector.connect(network, stream).await.unwrap();

    tls_stream
}

async fn start_irc_listener(app_handle: AppHandle) -> std::io::Result<()> {
    let mut message_text_buffer = String::from("");

    let tls_stream = connect_to_network("irc.libera.chat", 6697).await;
    let (mut read_tls_stream, mut write_tls_stream) = split(tls_stream);

    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut lines = BufReader::new(stdin).lines();

        while let Some(line) = lines.next_line().await.unwrap() {
            let formatted_line = format!("{}\r\n", line); // message must end with CRLF as per IRC protocol
            println!("{}", formatted_line);
            write_tls_stream
                .write_all(formatted_line.as_bytes())
                .await
                .expect("TODO: panic message");
        }
    });

    loop {
        let mut buffer = [0; 1024];
        read_tls_stream.read(&mut buffer).await?;

        let buffer_to_text = from_utf8(&buffer);

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
                    println!("{}", message);
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

            // this spawns a new OS thread
            std::thread::spawn(move || {
                // need a tokio runtine here, since the start_irc_listener is async
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async {
                    if let Err(e) = start_irc_listener(app_handle).await {
                        eprintln!("IRC listener encountered an error: {}", e);
                    }
                });
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
