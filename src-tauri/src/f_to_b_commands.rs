use crate::structs::{HistoryState, WriteTLSStreamState};
use tauri::State;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::MutexGuard;
use tokio_native_tls::TlsStream;

fn log_user_input(history_state: &State<HistoryState>, user_input_line: String) {
    history_state.history.lock().unwrap().push(user_input_line);
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

#[tauri::command]
pub async fn process_user_input_line(
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
