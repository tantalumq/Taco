use std::collections::HashMap;

use iced::{
    theme::{Button, Scrollable},
    widget::{button, column, container, row, scrollable, text_input},
    Command, Element, Length,
};
use structs::{
    requests::{
        ChatWithMembers, CreateChat, Session, WsChatMessage, WsDeleteMessage, WsMessageData,
    },
    Utc,
};

use crate::{server::server_get, server::server_post, ws_client::WsEvent};

use super::{
    chat::{Chat, ChatMessage},
    icon,
    letter_list::{LetterList, LetterListMessage},
    style_outline, ChatButtonStyle, ScrollableStyle,
};

pub struct ChatList {
    pub chats: HashMap<String, Chat>,
    pub client: reqwest::Client,
    pub username_input: String,
    pub session: Session,
    pub opened_chat: Option<String>,
    pub opened_chat_messages: LetterList,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatListMessage {
    Chat(ChatMessage, String),
    AddChat,
    ChatAdded(ChatWithMembers),
    UsernameInputChanged(String),
    MessagesLoaded(Vec<WsChatMessage>),
    LetterListMessage(LetterListMessage),
    Error(String),
}

impl ChatList {
    pub fn new(client: reqwest::Client, session: Session) -> Self {
        Self {
            chats: HashMap::new(),
            client: client.clone(),
            username_input: String::new(),
            session: session.clone(),
            opened_chat: None,
            opened_chat_messages: LetterList::new(client, None, session),
        }
    }

    pub fn update(&mut self, message: ChatListMessage) -> Command<ChatListMessage> {
        match message {
            ChatListMessage::Chat(msg, chat_id) => match msg {
                ChatMessage::OpenChat => {
                    if let Some(chat) = self.opened_chat.as_ref() {
                        self.chats.get_mut(chat).unwrap().is_open = false;
                    }

                    self.opened_chat = Some(chat_id.clone());
                    self.chats
                        .get_mut(self.opened_chat.as_ref().unwrap())
                        .unwrap()
                        .is_open = true;

                    self.opened_chat_messages.chat_id = Some(chat_id.clone());
                    self.opened_chat_messages.replying_to = None;
                    Command::perform(
                        server_get::<Vec<WsChatMessage>>(
                            self.client.clone(),
                            format!("messages/{chat_id}"),
                            Some(self.session.session_id.clone()),
                        ),
                        |msgs| ChatListMessage::MessagesLoaded(msgs.unwrap()),
                    )
                }
                msg => self
                    .chats
                    .get_mut(&chat_id)
                    .unwrap()
                    .update(msg)
                    .map(move |msg| ChatListMessage::Chat(msg, chat_id.clone())),
            },
            ChatListMessage::AddChat => {
                let command = Command::perform(
                    server_post::<ChatWithMembers>(
                        self.client.clone(),
                        "create_chat",
                        CreateChat {
                            other_members: self.username_input.clone(),
                        },
                        Some(self.session.session_id.clone()),
                    ),
                    |chat| match chat {
                        Ok(chat) => ChatListMessage::ChatAdded(chat),
                        Err(error) => ChatListMessage::Error(error.to_string()),
                    },
                );
                self.username_input = "".into();
                command
            }

            ChatListMessage::ChatAdded(chat) => {
                let (cmd, id) = Chat::new(
                    self,
                    self.session.user_id.clone(),
                    chat.id,
                    chat.members,
                    chat.last_updated,
                );
                cmd.map(move |msg| ChatListMessage::Chat(msg, id.clone()))
            }
            ChatListMessage::UsernameInputChanged(user_id) => {
                self.username_input = user_id;
                Command::none()
            }
            ChatListMessage::MessagesLoaded(messages) => {
                self.opened_chat_messages.clear();
                for msg in messages {
                    self.opened_chat_messages.add_message(msg);
                }
                Command::none()
            }
            ChatListMessage::LetterListMessage(msg) => match msg {
                LetterListMessage::MessageSent { .. }
                | LetterListMessage::MessageDeleted { .. } => {
                    self.chats
                        .get_mut(self.opened_chat.as_ref().unwrap())
                        .unwrap()
                        .last_updated = Utc::now();

                    self.opened_chat_messages
                        .update(msg)
                        .map(|msg| ChatListMessage::LetterListMessage(msg))
                }
                LetterListMessage::WsEvent(WsEvent::Message(WsMessageData::CreateChat(chat))) => {
                    self.update(ChatListMessage::ChatAdded(ChatWithMembers {
                        id: chat.chat_id,
                        members: chat.members,
                        last_updated: Utc::now(),
                    }))
                }
                LetterListMessage::WsEvent(WsEvent::Message(ref ws_msg)) => {
                    match ws_msg {
                        WsMessageData::ChatMessage(WsChatMessage { chat_id, .. })
                        | WsMessageData::DeleteMessage(WsDeleteMessage { chat_id, .. }) => {
                            self.chats.get_mut(chat_id).unwrap().last_updated = Utc::now();
                        }
                        _ => {}
                    }
                    self.opened_chat_messages
                        .update(msg)
                        .map(|msg| ChatListMessage::LetterListMessage(msg))
                }
                LetterListMessage::ChatDeleted(chat) => {
                    if self
                        .opened_chat
                        .as_ref()
                        .is_some_and(|c| c == &chat.chat_id)
                    {
                        self.opened_chat = None;
                        self.opened_chat_messages.chat_id = None;
                    }
                    self.chats.remove(&chat.chat_id);
                    Command::none()
                }
                _ => self.opened_chat_messages.update(msg).map(|msg| {
                    if let LetterListMessage::Error(err) = msg {
                        ChatListMessage::Error(err)
                    } else {
                        ChatListMessage::LetterListMessage(msg)
                    }
                }),
            },
            _ => Command::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<ChatListMessage> {
        self.opened_chat_messages
            .subscription()
            .map(|msg| ChatListMessage::LetterListMessage(msg))
    }

    pub fn view(&self, current_user_id: String) -> Element<ChatListMessage> {
        let mut chats = self
            .chats
            .iter()
            .map(|chat| {
                (
                    chat.1
                        .view(current_user_id.clone())
                        .map(|msg| ChatListMessage::Chat(msg, chat.0.clone())),
                    chat.1.last_updated,
                )
            })
            .collect::<Vec<_>>();

        chats.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));

        let mut chat_list_letter_list = row![container(
            column![
                row![
                    text_input("Имя пользователя", &self.username_input)
                        .padding(8)
                        .on_input(ChatListMessage::UsernameInputChanged),
                    button(icon(''))
                        .padding([8, 16])
                        .style(Button::Custom(Box::new(ChatButtonStyle::SenderMessage)))
                        .on_press(ChatListMessage::AddChat),
                ]
                .spacing(5),
                scrollable(
                    container(
                        column(chats.into_iter().rev().map(|(chat, _)| chat).collect()).spacing(5)
                    )
                    .padding([0, 10, 0, 0]),
                )
                .style(Scrollable::Custom(Box::new(ScrollableStyle))),
            ]
            .max_width(350)
            .spacing(5)
        )
        .height(Length::Fill)
        .padding(10)
        .style(style_outline)]
        .padding(10)
        .width(Length::Fill)
        .spacing(20);
        if let Some(opened_chat) = &self.opened_chat {
            chat_list_letter_list = chat_list_letter_list.push(
                container(
                    self.opened_chat_messages
                        .view(
                            self.chats.get(opened_chat).unwrap().members.clone(),
                            current_user_id,
                        )
                        .map(|msg| ChatListMessage::LetterListMessage(msg)),
                )
                .style(style_outline)
                .padding(10),
            );
        }
        chat_list_letter_list.into()
    }
}
