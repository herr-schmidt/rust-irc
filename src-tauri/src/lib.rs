use regex::{Match, Regex};
use std::str::from_utf8;
use std::sync::{Arc, LazyLock};
use std::{collections::HashMap, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::MutexGuard;
use tokio_native_tls::{native_tls, TlsConnector, TlsStream};

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    nickname: String,
    real_name: String,
}

struct ChannelData {
    users: Vec<String>,
}

struct HistoryState {
    history: Mutex<Vec<String>>,
}

struct ChannelsState {
    channels_data: Mutex<HashMap<String, ChannelData>>,
}

struct WriteTLSStreamState {
    write_tls_stream: Arc<tokio::sync::Mutex<WriteHalf<TlsStream<TcpStream>>>>,
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

#[derive(Debug)]
enum MessageType {
    RPLTopic = 332,
}

#[derive(Debug)]
struct Message {
    message_type: MessageType,
    description: Option<String>,
    prefix: Option<String>,
    command: Option<String>,
    parameters: Option<String>,
    trailing: Option<String>,
}

fn extract_match(capture: Option<Match>) -> Option<String> {
    let match_option = match capture {
        None => None,
        Some(_regex_match) => Some(capture.unwrap().as_str().to_string()),
    };
    return match_option;
}

fn parse_message(message: &str) -> Message {
    static SERVER_RESPONSE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^((:[^\ ]*))?(\ ([^\ :]+))(\ ([^\:]*))?(\ (:.*))?$").unwrap()
    });
    let parsed_message: Message = match SERVER_RESPONSE_REGEX.captures(message) {
        None => Message {
            message_type: MessageType::RPLTopic,
            description: None,
            prefix: None,
            command: None,
            parameters: None,
            trailing: None,
        },
        Some(captures) => {
            let prefix = extract_match(captures.get(2));
            let command = extract_match(captures.get(4));
            let parameters = extract_match(captures.get(6));
            let trailing = extract_match(captures.get(8));

            return Message {
                message_type: MessageType::RPLTopic,
                description: None,
                prefix,
                command,
                parameters,
                trailing,
            };
        }
    };

    return parsed_message;
}

fn log_user_input(history_state: &State<HistoryState>, user_input_line: String) {
    history_state.history.lock().unwrap().push(user_input_line);
}

#[tauri::command]
async fn process_user_input_line(
    history_state: State<'_, HistoryState>,
    write_tls_stream_state: State<'_, WriteTLSStreamState>,
    user_input_line: String,
) -> Result<(), String> {
    log_user_input(&history_state, user_input_line.clone());
    send_command(
        write_tls_stream_state.write_tls_stream.lock().await,
        user_input_line,
    )
    .await;

    Ok(())
}

async fn send_command(
    mut write_tls_stream: MutexGuard<'_, WriteHalf<TlsStream<TcpStream>>>,
    command: String,
) {
    let formatted_line = format!("{}\r\n", command); // message must end with CRLF as per IRC protocol

    write_tls_stream
        .write_all(formatted_line.as_bytes())
        .await
        .expect("TODO: panic message");
}

async fn start_irc_listener(app_handle: AppHandle) -> std::io::Result<()> {
    let mut message_text_buffer = String::from("");
    let tls_stream = connect_to_network("irc.libera.chat", 6697).await;
    let (mut read_tls_stream, mut write_tls_stream) = split(tls_stream);

    app_handle.manage(WriteTLSStreamState {
        write_tls_stream: Arc::new(tokio::sync::Mutex::new(write_tls_stream)),
    });

    loop {
        let mut buffer = [0; 1024];
        let bytes_read = read_tls_stream.read(&mut buffer).await?;

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
                        println!("{:?}", parse_message(&String::from(message).to_string()));

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
                let runtime = tokio::runtime::Builder::new_current_thread()
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
        .invoke_handler(tauri::generate_handler![greet, process_user_input_line])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
