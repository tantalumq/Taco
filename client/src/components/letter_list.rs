use crate::{
    components::{truncate_message, ChatButtonStyle},
    server_get, server_post,
};

use super::{
    chat::Chat,
    letter::{Letter, LetterMessage},
    ScrollableStyle,
};
use iced::{
    alignment,
    theme::{Button, Scrollable},
    widget::{
        button, column, row, scrollable, scrollable::RelativeOffset, text, text_input, Space,
    },
    Command, Element, Length,
};
use indexmap::IndexMap;

use structs::requests::{CreateMessage, DeleteMessage, Session, WsChatMessage};
use structs::{DateTime, FixedOffset, Utc};

#[derive(Clone)]
pub struct LetterList {
    pub messages: IndexMap<String, Letter>,
    pub message_input: String,
    pub client: reqwest::Client,
    pub chat_id: Option<String>,
    pub session: Session,
    pub replying_to: Option<String>,
    pub scrollable_id: scrollable::Id,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LetterListMessage {
    LetterMessage(LetterMessage, String),
    MessageInputChanged(String),
    SendPressed,
    MessageSent { id: String, message: String },
    CancelReply,
    MessageDeleted(String),
}

impl LetterList {
    pub fn new(client: reqwest::Client, chat_id: Option<String>, session: Session) -> Self {
        Self {
            messages: IndexMap::new(),
            message_input: String::new(),
            client,
            chat_id,
            session,
            replying_to: None,
            scrollable_id: scrollable::Id::unique(),
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn add_message(&mut self, chat_message: WsChatMessage) {
        self.messages
            .insert(chat_message.message_id.clone(), Letter(chat_message));
    }

    pub fn update(&mut self, message: LetterListMessage) -> Command<LetterListMessage> {
        match message {
            LetterListMessage::LetterMessage(msg, id) => {
                match msg {
                    LetterMessage::ReplyStarted => {
                        self.replying_to = Some(id);
                        Command::none()
                    }
                    LetterMessage::LetterDelete => Command::perform(
                        server_post::<()>(
                            self.client.clone(),
                            "delete_message",
                            DeleteMessage { id: id.clone() },
                            Some(self.session.session_id.clone()),
                        ),
                        move |_| LetterListMessage::MessageDeleted(id),
                    ),
                    _ => Command::none(),
                }
                //self.messages.get_mut(&id).unwrap().update(msg);
            }
            LetterListMessage::MessageInputChanged(value) => {
                self.message_input = value;
                Command::none()
            }
            LetterListMessage::SendPressed => {
                let message = self.message_input.clone();
                self.message_input = String::new();
                let client = self.client.clone();
                let reply_to_id = self.replying_to.clone();
                Command::perform(
                    server_post::<String>(
                        client,
                        "create_message",
                        CreateMessage {
                            chat_id: self.chat_id.clone().unwrap(),
                            content: message.clone(),
                            reply_to_id,
                        },
                        Some(self.session.session_id.clone()),
                    ),
                    move |msg| LetterListMessage::MessageSent {
                        id: msg.unwrap(),
                        message,
                    },
                )
            }
            LetterListMessage::MessageSent { id, message } => {
                self.add_message(WsChatMessage {
                    message_id: id,
                    message,
                    sender_id: self.session.user_id.clone(),
                    chat_id: self.chat_id.as_ref().unwrap().clone(),
                    reply_to: self.replying_to.clone(),
                    created_at: Utc::now(),
                });
                self.replying_to = None;
                scrollable::snap_to(self.scrollable_id.clone(), RelativeOffset::END)
            }
            LetterListMessage::CancelReply => {
                self.replying_to = None;
                Command::none()
            }
            LetterListMessage::MessageDeleted(id) => {
                self.messages.remove(&id);
                Command::none()
            }
        }
    }
    pub fn view(
        &self,
        members: Vec<String>,
        current_user_id: String,
    ) -> Element<LetterListMessage> {
        let other_member = Chat::get_other_member(current_user_id.clone(), &members);
        let nickname_text = text(other_member).size(25);

        let message_send_column = if let Some(replying_to) = self.replying_to.clone() {
            let message = self.messages.get(&replying_to).unwrap().0.clone();
            let content = message.message;
            column![row![
                text(&format!(
                    "↱ {}: {}",
                    message.sender_id,
                    truncate_message(content)
                )),
                Space::with_width(Length::Fill),
                button("×")
                    .padding([0, 10])
                    .style(Button::Custom(Box::new(ChatButtonStyle::Delete)))
                    .on_press(LetterListMessage::CancelReply)
            ]
            .spacing(5)]
        } else {
            column![]
        };

        column![
            nickname_text
                .width(Length::Fill)
                .vertical_alignment(alignment::Vertical::Center)
                .horizontal_alignment(alignment::Horizontal::Center),
            scrollable(
                column(
                    self.messages
                        .iter()
                        .map(|message| {
                            message
                                .1
                                .view(self, current_user_id.clone())
                                .map(|msg| LetterListMessage::LetterMessage(msg, message.0.clone()))
                        })
                        .collect()
                )
                .width(Length::Fill)
                .spacing(5)
                .padding(5)
            )
            .id(self.scrollable_id.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(Scrollable::Custom(Box::new(ScrollableStyle))),
            message_send_column.spacing(8).push(
                row![
                    text_input("Message", &self.message_input)
                        .padding(8)
                        .on_input(|value| LetterListMessage::MessageInputChanged(value))
                        .on_submit(LetterListMessage::SendPressed),
                    button("Send")
                        .padding([8, 18])
                        .style(Button::Custom(Box::new(ChatButtonStyle::SenderMessage)))
                        .on_press(LetterListMessage::SendPressed)
                ]
                .spacing(8)
            )
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(5)
        .padding(5)
        .into()
    }
}
