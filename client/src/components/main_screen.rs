use iced::{widget::container, Length};
use structs::requests::{ChatWithMembers, Session};

use crate::server;

use super::{
    chat::{Chat, ChatMessage},
    chat_list::{ChatList, ChatListMessage},
};

pub struct MainScreen {
    session: Session,
    client: reqwest::Client,
    chat_list: ChatList,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MainScreenMessage {
    ChatsLoaded(Vec<ChatWithMembers>),
    ChatListMessage(ChatListMessage),
    Error(String),
}

impl MainScreen {
    pub fn new(
        session: Session,
        client: reqwest::Client,
    ) -> (Self, iced::Command<MainScreenMessage>) {
        let screen = Self {
            session,
            client,
            chat_list: ChatList::new(client.clone(), session.clone()),
        };
        (
            screen,
            iced::Command::perform(
                server::server_get::<Vec<ChatWithMembers>>(
                    client.clone(),
                    "chats".into(),
                    Some(session.session_id),
                ),
                move |chats| MainScreenMessage::ChatsLoaded(chats.unwrap()),
            ),
        )
    }

    pub fn update(&self, message: MainScreenMessage) -> iced::Command<MainScreenMessage> {
        match message {
            MainScreenMessage::ChatListMessage(msg) => self.chat_list.update(msg).map(|msg| {
                if let ChatListMessage::Error(err) = msg {
                    MainScreenMessage::Error(err)
                } else {
                    MainScreenMessage::ChatListMessage(msg)
                }
            }),
            MainScreenMessage::ChatsLoaded(loaded_chats) => {
                let chats: Vec<(iced::Command<ChatMessage>, String)> = loaded_chats
                    .into_iter()
                    .map(|chat| {
                        Chat::new(
                            &mut self.chat_list,
                            self.session.clone().user_id,
                            chat.id,
                            chat.members,
                            chat.last_updated,
                        )
                    })
                    .collect();
                iced::Command::batch(chats.into_iter().map(|(cmd, chat_id)| {
                    cmd.map(move |msg| {
                        MainScreenMessage::ChatListMessage(ChatListMessage::ChatMessage(
                            msg,
                            chat_id.clone(),
                        ))
                    })
                }))
            }
            _ => iced::Command::none(),
        }
    }

    pub fn view(&self) -> iced::Element<MainScreenMessage> {
        container(
            self.chat_list
                .view(self.session.user_id.clone())
                .map(MainScreenMessage::ChatListMessage),
        )
        .width(Length::Fill)
        .padding(10)
        .into()
    }
}
