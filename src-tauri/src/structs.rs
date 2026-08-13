use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

#[derive(Debug)]
pub(crate) enum MessageType {
    RPL_TOPIC = 332,
    RPL_NAMREPLY = 353,
}

#[derive(Debug)]
pub(crate) struct Message {
    pub(crate) message_type: Option<MessageType>,
    pub(crate) description: Option<String>,
    pub(crate) prefix: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) parameters: Option<String>,
    pub(crate) trailing: Option<String>,
}

pub(crate) struct HistoryState {
    pub(crate) history: Mutex<Vec<String>>,
}

pub(crate) struct ChannelsState {
    pub(crate) channels_data: Mutex<HashMap<String, ChannelData>>,
}

pub(crate) struct WriteTLSStreamState {
    pub(crate) write_tls_stream: Arc<tokio::sync::Mutex<WriteHalf<TlsStream<TcpStream>>>>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub(crate) struct User {
    pub(crate) nickname: String,
    pub(crate) real_name: Option<String>,
}

pub(crate) struct ChannelData {
    pub(crate) users: Vec<String>,
}
