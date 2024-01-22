use super::{
    chat::Chat,
    icon_button,
    letter::{Letter, LetterMessage},
    ScrollableStyle,
};
use crate::{
    components::{truncate_message, ChatButtonStyle},
    server::server_post,
    ws_client,
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

use structs::requests::{
    CreateMessage, DeleteMessage, LeaveChat, Session, WsChatMessage, WsLeaveChat, WsMessageData,
};
use structs::Utc;

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
    MessageSent {
        id: String,
        message: String,
        sender: String,
        reply_to: Option<String>,
    },
    CancelReply,
    MessageDeleted(String),
    WsEvent(ws_client::WsEvent),
    ChatDelete,
    ChatDeleted(WsLeaveChat),
    Error(String),
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
                }
                //self.messages.get_mut(&id).unwrap().update(msg);
            }
            LetterListMessage::MessageInputChanged(value) => {
                self.message_input = truncate_message(value, 300);
                Command::none()
            }
            LetterListMessage::SendPressed => {
                let message = self.message_input.clone();
                self.message_input = String::new();
                let client = self.client.clone();
                let reply_to_id = self.replying_to.clone();
                let sender = self.session.user_id.clone();
                self.replying_to = None;
                Command::perform(
                    server_post::<String>(
                        client,
                        "create_message",
                        CreateMessage {
                            chat_id: self.chat_id.clone().unwrap(),
                            content: message.clone(),
                            reply_to_id: reply_to_id.clone(),
                        },
                        Some(self.session.session_id.clone()),
                    ),
                    move |msg| LetterListMessage::MessageSent {
                        id: msg.unwrap(),
                        message,
                        sender,
                        reply_to: reply_to_id,
                    },
                )
            }
            LetterListMessage::MessageSent {
                id,
                message,
                sender,
                reply_to,
            } => {
                self.add_message(WsChatMessage {
                    message_id: id,
                    message,
                    sender_id: sender,
                    chat_id: self.chat_id.as_ref().unwrap().clone(),
                    reply_to,
                    created_at: Utc::now(),
                });
                scrollable::snap_to(self.scrollable_id.clone(), RelativeOffset::END)
            }
            LetterListMessage::CancelReply => {
                self.replying_to = None;
                Command::none()
            }
            LetterListMessage::MessageDeleted(id) => {
                if let Some(reply) = &self.replying_to {
                    if reply == &id {
                        self.replying_to = None;
                    }
                }
                self.messages.remove(&id);
                Command::none()
            }
            LetterListMessage::WsEvent(ws_client::WsEvent::Message(data)) => match data {
                WsMessageData::ChatMessage(msg) => {
                    if let Some(chat) = &self.chat_id {
                        if chat == &msg.chat_id {
                            return self.update(LetterListMessage::MessageSent {
                                id: msg.message_id,
                                message: msg.message,
                                sender: msg.sender_id,
                                reply_to: msg.reply_to,
                            });
                        }
                    }
                    Command::none()
                }
                WsMessageData::DeleteMessage(msg) => {
                    self.update(LetterListMessage::MessageDeleted(msg.message_id))
                }
                _ => Command::none(),
            },
            LetterListMessage::ChatDelete => {
                let member = self.session.user_id.clone();
                let chat_id = self.chat_id.clone().unwrap();
                Command::perform(
                    server_post::<()>(
                        self.client.clone(),
                        "leave_chat",
                        LeaveChat {
                            chat_id: chat_id.clone(),
                        },
                        Some(self.session.session_id.clone()),
                    ),
                    |result| match result {
                        Ok(_) => LetterListMessage::ChatDeleted(WsLeaveChat { chat_id, member }),
                        Err(err) => LetterListMessage::Error(err.to_string()),
                    },
                )
            }
            _ => Command::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<LetterListMessage> {
        ws_client::connect(self.session.session_id.clone())
            .map(|event| LetterListMessage::WsEvent(event))
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
                    truncate_message(content, 80)
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
            row![
                nickname_text
                    .width(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
                icon_button('').on_press(LetterListMessage::ChatDelete)
            ],
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
                    text_input("Сообщение", &self.message_input)
                        .padding(8)
                        .on_input(|value| LetterListMessage::MessageInputChanged(value))
                        .on_submit(LetterListMessage::SendPressed),
                    icon_button('')
                        .padding([8, 14])
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
