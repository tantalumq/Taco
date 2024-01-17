use std::collections::HashMap;

use iced::{
    theme::{Button, Scrollable},
    widget::{button, column, container, row, scrollable, text, text_input},
    Command, Element, Length,
};
use structs::{
    requests::{ChatWithMembers, CreateChat, Session, WsChatMessage},
    Utc,
};

use crate::{server_get, server_post};

use super::{
    chat::{Chat, ChatMessage},
    letter_list::{LetterList, LetterListMessage},
    style_outline, ChatButtonStyle, ScrollableStyle, ICON_FONT,
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
    ChatMessage(ChatMessage, String),
    AddChat,
    ChatAdded(ChatWithMembers),
    UsernameInputChanged(String),
    MessagesLoaded(Vec<WsChatMessage>),
    LetterListMessage(LetterListMessage),
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
            ChatListMessage::ChatMessage(msg, chat_id) => {
                self.chats.get_mut(&chat_id).unwrap().update(msg.clone());
                match msg {
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
                    _ => Command::none(),
                }
            }
            ChatListMessage::AddChat => Command::perform(
                server_post::<ChatWithMembers>(
                    self.client.clone(),
                    "create_chat",
                    CreateChat {
                        other_members: self.username_input.clone(),
                    },
                    Some(self.session.session_id.clone()),
                ),
                |chat| ChatListMessage::ChatAdded(chat.unwrap()),
            ),
            ChatListMessage::ChatAdded(chat) => {
                let (cmd, id) = Chat::new(
                    self,
                    self.session.user_id.clone(),
                    chat.id,
                    chat.members,
                    chat.last_updated,
                );
                cmd.map(move |msg| ChatListMessage::ChatMessage(msg, id.clone()))
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
            ChatListMessage::LetterListMessage(msg) => {
                match msg {
                    LetterListMessage::MessageSent { id: _, message: _ } => {
                        self.chats
                            .get_mut(self.opened_chat.as_ref().unwrap())
                            .unwrap()
                            .last_updated = Utc::now();
                    }
                    _ => {}
                }
                self.opened_chat_messages
                    .update(msg)
                    .map(|msg| ChatListMessage::LetterListMessage(msg))
            }
        }
    }

    pub fn view(&self, current_user_id: String) -> Element<ChatListMessage> {
        let mut chats = self
            .chats
            .iter()
            .map(|chat| {
                (
                    chat.1
                        .view(current_user_id.clone())
                        .map(|msg| ChatListMessage::ChatMessage(msg, chat.0.clone())),
                    chat.1.last_updated,
                )
            })
            .collect::<Vec<_>>();

        chats.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));

        let mut chat_list_letter_list = row![container(
            column![
                row![
                    text_input("Username", &self.username_input)
                        .padding(8)
                        .on_input(ChatListMessage::UsernameInputChanged),
                    button(text("").font(ICON_FONT))
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
