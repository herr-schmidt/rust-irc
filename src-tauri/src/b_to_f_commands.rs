use crate::parsing::extract_channel_users_vector;
use crate::structs::{ChannelsState, Message, User};
use tauri::{AppHandle, Emitter, Manager};

pub fn update_users(application_handler: &AppHandle, message_struct: &Message) {
    let users: Vec<User> = extract_channel_users_vector(message_struct);
    let channels_state = application_handler.state::<ChannelsState>();
    application_handler.emit("new-users", users).unwrap();
}
