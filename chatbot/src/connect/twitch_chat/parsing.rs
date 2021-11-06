use std::str::FromStr;

#[derive(Debug)]
pub enum ParseMessageTypeError {
    UnknownMessageType,
}

#[derive(Debug)]
pub struct MessageInfo {
    pub user: String,
    pub text: String,
}

#[derive(Debug)]
pub enum MessageType {
    UserMessage(MessageInfo),
    PingMessage(String),
}

impl MessageType {
    fn parse_from_str(s: &str) -> Result<Self, ParseMessageTypeError> {
        if s.starts_with(':') {
            MessageType::from_text_message(s)
        } else {
            MessageType::from_ping_message(s)
        }
    }

    // Example message: PING :tmi.twitch.tv
    fn from_ping_message(raw_message: &str) -> Result<Self, ParseMessageTypeError> {
        if let Some(server) = raw_message.strip_prefix("PING :") {
            Ok(MessageType::PingMessage(server.to_owned()))
        } else {
            Err(ParseMessageTypeError::UnknownMessageType)
        }
    }

    // Example message: :carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :backseating backseating
    fn from_text_message(raw_message: &str) -> Result<Self, ParseMessageTypeError> {
        enum ParsingState {
            UserName,
            AdditionalUserInfo,
            MessageToken,
            Channel,
            MessageText,
        }
        use ParsingState::*;

        let mut state = UserName;
        let mut user_name = &raw_message[0..0];
        let mut marker = 0;

        for (i, codepoint) in raw_message.char_indices() {
            match state {
                // :carkhy!carkhy@carkhy.tmi.twitch.tv
                UserName => match codepoint {
                    ':' => marker = i + 1,
                    ' ' => return Err(ParseMessageTypeError::UnknownMessageType),
                    '!' => {
                        user_name = &raw_message[marker..i];
                        state = AdditionalUserInfo;
                    }
                    _ => (),
                },
                AdditionalUserInfo => {
                    if codepoint == ' ' {
                        marker = i + 1;
                        state = MessageToken
                    }
                }
                // PRIVMSG #captaincallback :backseating backseating
                MessageToken => {
                    if codepoint == ' ' {
                        let token = &raw_message[marker..i];
                        if token == "PRIVMSG" {
                            state = Channel;
                        } else {
                            // we're only interested in PRIVMSG
                            return Err(ParseMessageTypeError::UnknownMessageType);
                        }
                    }
                }
                Channel => {
                    if codepoint == ' ' {
                        state = MessageText;
                    }
                }
                MessageText => {
                    return Ok(MessageType::UserMessage(MessageInfo {
                        user: user_name.to_owned(),
                        text: raw_message[(i + 1)..].trim().to_owned(),
                    }));
                }
            }
        }
        Err(ParseMessageTypeError::UnknownMessageType)
    }
}

impl FromStr for MessageType {
    type Err = ParseMessageTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MessageType::parse_from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_private_messages() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message";
        let parsed = MessageType::from_str(raw_message);
        assert!(parsed.is_ok());
        if let MessageType::UserMessage(info) = parsed.unwrap() {
            assert_eq!(info.user, "carkhy");
            assert_eq!(
                info.text,
                "a function that takes a string and returns the message"
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_private_messages_with_trailing_newlines() {
        let raw_message = ":carkhy!carkhy@carkhy.tmi.twitch.tv PRIVMSG #captaincallback :a function that takes a string and returns the message\n";
        let parsed = MessageType::from_str(raw_message);
        assert!(parsed.is_ok());
        if let MessageType::UserMessage(info) = parsed.unwrap() {
            assert_eq!(info.user, "carkhy");
            assert_eq!(
                info.text,
                "a function that takes a string and returns the message"
            );
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_ping_messages() {
        let ping_message = "PING :tmi.twitch.tv";
        let parsed = MessageType::from_str(ping_message);
        assert!(parsed.is_ok());
        if let MessageType::PingMessage(server) = parsed.unwrap() {
            assert_eq!(server, "tmi.twitch.tv");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn collect_after_skipping_past_the_end() {
        let s = String::from("bleh");
        let iter = s.chars().skip(35);
        let s2: String = iter.collect();
        assert_eq!(s2, "");
    }

    #[test]
    fn slice_starting_at_len() {
        let s = String::from("bleh");
        let slice = &s[s.len()..];
        assert_eq!(slice, "");
    }
}