use std::str::from_utf8;
use std::sync::LazyLock;
use tauri::AppHandle;
use tokio::io::{self, split, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls, TlsConnector, TlsStream};
use regex::Regex;

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

enum MessageType {
    RPLTopic = 332,
}

struct Message {
    message_type: MessageType,
    description: String,
    prefix: String,
    command: String,
    parameters: String,
    trailing: String
}

fn parse_message(message: &str) -> String {
    static SERVER_RESPONSE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^((:[^\ ]*))?(\ ([^\ :]+))(\ ([^\:]*))?(\ (:.*))?$").unwrap());
    let clean_message = message.trim_matches(['\r', '\n']);
    println!("{}", message);
    println!("{}", clean_message);
    
    let parsed_message = match SERVER_RESPONSE_REGEX.captures(clean_message) {
        None => String::from(format!("ERROR")).to_string(),
        Some(captures) => {
            
            let prefix_match = captures.get(2);
            let prefix = match prefix_match {
                None => String::from("ERROR").to_string(),
                Some(_regex_match) => prefix_match.unwrap().as_str().to_string()
            };
            let command_match = captures.get(4);
            let command = match command_match {
                None => String::from("ERROR").to_string(),
                Some(_regex_match) => command_match.unwrap().as_str().to_string()
            };
            let parameters_match = captures.get(6);
            let parameters = match parameters_match {
                None => String::from("ERROR").to_string(),
                Some(_regex_match) => parameters_match.unwrap().as_str().to_string()
            };
            let trailing_match = captures.get(8);
            let trailing = match trailing_match {
                None => String::from("ERROR").to_string(),
                Some(_regex_match) => trailing_match.unwrap().as_str().to_string()
            };
            
            return String::from(format!("PREFIX: {}\nCOMMAND: {}\nPARAMETERS: {}\nTRAILING: {}", prefix, command, parameters, trailing)).to_string();
        }
    };
    
    return parsed_message
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
        let mut buffer = [0; 4096];
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
                    println!("{}", message);
                    println!("{}", parse_message(&String::from(message).to_string()));
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
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
