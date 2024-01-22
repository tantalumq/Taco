use iced::{widget::column, Length};
use structs::requests::{ChatWithMembers, Session};

use crate::server;

use super::{
    chat::{Chat, ChatMessage},
    chat_list::{ChatList, ChatListMessage},
    header::{Header, HeaderMessage},
};

pub struct MainScreen {
    session: Session,
    header: Header,
    chat_list: ChatList,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MainScreenMessage {
    ChatsLoaded(Vec<ChatWithMembers>),
    ChatList(ChatListMessage),
    Header(HeaderMessage),
    Error(String),
}

impl MainScreen {
    pub fn new(
        session: Session,
        client: reqwest::Client,
    ) -> (Self, iced::Command<MainScreenMessage>) {
        let (header, load_header_pfp) = Header::new(session.clone(), client.clone());
        let screen = Self {
            session: session.clone(),
            chat_list: ChatList::new(client.clone(), session.clone()),
            header,
        };
        (
            screen,
            iced::Command::batch(vec![
                iced::Command::perform(
                    server::server_get::<Vec<ChatWithMembers>>(
                        client,
                        "chats".into(),
                        Some(session.session_id.clone()),
                    ),
                    move |chats| MainScreenMessage::ChatsLoaded(chats.unwrap()),
                ),
                load_header_pfp.map(MainScreenMessage::Header),
            ]),
        )
    }

    pub fn update(&mut self, message: MainScreenMessage) -> iced::Command<MainScreenMessage> {
        match message {
            MainScreenMessage::Header(msg) => {
                self.header.update(msg).map(MainScreenMessage::Header)
            }
            MainScreenMessage::ChatList(msg) => self.chat_list.update(msg).map(|msg| {
                if let ChatListMessage::Error(err) = msg {
                    MainScreenMessage::Error(err)
                } else {
                    MainScreenMessage::ChatList(msg)
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
                        MainScreenMessage::ChatList(ChatListMessage::Chat(msg, chat_id.clone()))
                    })
                }))
            }
            _ => iced::Command::none(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<MainScreenMessage> {
        self.chat_list
            .subscription()
            .map(MainScreenMessage::ChatList)
    }

    pub fn view(&self) -> iced::Element<MainScreenMessage> {
        column![
            self.header.view().map(MainScreenMessage::Header),
            self.chat_list
                .view(self.session.user_id.clone())
                .map(MainScreenMessage::ChatList),
        ]
        .width(Length::Fill)
        .into()
    }
}
