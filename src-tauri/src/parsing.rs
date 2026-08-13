use crate::structs::{Message, MessageType, User};
use regex::{Match, Regex};
use std::sync::LazyLock;

fn extract_match(capture: Option<Match>) -> Option<String> {
    let match_option = match capture {
        None => None,
        Some(_regex_match) => Some(capture.unwrap().as_str().to_string()),
    };
    return match_option;
}

fn extract_message_type(command: &Option<String>) -> Option<MessageType> {
    return match command.as_deref() {
        Some("332") => Some(MessageType::RPL_TOPIC),
        Some("353") => Some(MessageType::RPL_NAMREPLY),
        _ => None,
    };
}

pub(crate) fn extract_channel_users_vector(message: &Message) {
    match message.trailing.as_deref() {
        Some(trailing) => {
            println!("TRAILING: {}", trailing)
        }
        None => {
            println!("Nothing")
        }
    };
}

pub(crate) fn parse_message(message: &str) -> Message {
    static SERVER_RESPONSE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^((:[^\ ]*))?(\ ([^\ :]+))(\ ([^\:]*))?(\ (:.*))?$").unwrap()
    });
    let parsed_message: Message = match SERVER_RESPONSE_REGEX.captures(message) {
        None => Message {
            message_type: None,
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
                message_type: extract_message_type(&command),
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
