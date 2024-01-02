use std::collections::HashMap;

use iced::{
    alignment,
    theme::Button,
    widget::{button, column, container, image, row, scrollable, text, text_input, Image},
    Command, Element, Length,
};
use structs::requests::{ChatWithMembers, CreateChat, Session};

use crate::{get_profile_picture, server_get, server_post, AppMessage};

pub struct ChatList {
    pub chats: HashMap<String, Chat>,
    pub client: reqwest::Client,
    pub username_input: String,
    pub session: Session,
    pub opened_chat: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatListMessage {
    ChatMessage(ChatMessage, String),
    AddChat,
    ChatAdded(ChatWithMembers),
    UsernameInputChanged(String),
}

impl ChatList {
    pub fn new(client: reqwest::Client, session: Session) -> Self {
        Self {
            chats: HashMap::new(),
            client,
            username_input: String::new(),
            session,
            opened_chat: None,
        }
    }

    pub fn update(&mut self, message: ChatListMessage) -> Command<ChatListMessage> {
        match message {
            ChatListMessage::ChatMessage(msg, chat_id) => {
                match msg {
                    ChatMessage::OpenChat => self.opened_chat = Some(chat_id.clone()),
                    _ => {}
                };
                self.chats.get_mut(&chat_id).unwrap().update(msg);
                Command::none()
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
                let (cmd, id) =
                    Chat::new(self, self.session.user_id.clone(), chat.id, chat.members);
                cmd.map(move |msg| ChatListMessage::ChatMessage(msg, id.clone()))
            }
            ChatListMessage::UsernameInputChanged(user_id) => {
                self.username_input = user_id;
                Command::none()
            }
        }
    }

    pub fn view(&self, current_user_id: String) -> Element<ChatListMessage> {
        column![
            row![
                text_input("Username", &self.username_input)
                    .on_input(ChatListMessage::UsernameInputChanged),
                button("add chat")
                    .style(Button::Secondary)
                    .on_press(ChatListMessage::AddChat),
            ]
            .spacing(5),
            scrollable(
                column(
                    self.chats
                        .iter()
                        .map(|chat| {
                            chat.1
                                .view(current_user_id.clone())
                                .map(|msg| ChatListMessage::ChatMessage(msg, chat.0.clone()))
                        })
                        .collect(),
                )
                .width(Length::Fill)
                .spacing(5)
                .padding(10),
            )
        ]
        .spacing(5)
        .into()
    }
}

#[derive(Clone)]
pub struct Chat {
    pub id: String,
    pub members: Vec<String>,
    pub message_list: LetterList,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatMessage {
    OpenChat,
    ProfilePictureLoaded(Option<String>),
}

impl Chat {
    pub fn new(
        chat_list: &mut ChatList,
        current_user_id: String,
        id: String,
        members: Vec<String>,
    ) -> (Command<ChatMessage>, String) {
        chat_list.chats.insert(
            id.clone(),
            Self {
                id: id.clone(),
                members: members.clone(),
                message_list: LetterList {
                    messages: HashMap::new(),
                    client: chat_list.client.clone(),
                    session: chat_list.session.clone(),
                },
                profile_picture: None,
            },
        );

        let client = chat_list.client.clone();
        (
            Command::perform(
                get_profile_picture(client, Chat::get_other_member(current_user_id, &members)),
                |pfp| ChatMessage::ProfilePictureLoaded(pfp),
            ),
            id,
        )
    }

    pub fn update(&mut self, message: ChatMessage) {
        match message {
            ChatMessage::ProfilePictureLoaded(pfp) => self.profile_picture = pfp,
            ChatMessage::OpenChat => {}
            _ => todo!(),
        }
    }

    pub fn get_other_member(current_user_id: String, members: &[String]) -> String {
        let first_member = members.get(0);
        let members = match (first_member, members.get(1).or(first_member)) {
            (Some(a), Some(b)) => (a.clone(), b.clone()),
            _ => panic!("group chats are not supported"),
        };

        let other_member = if members.0 == current_user_id {
            members.1
        } else {
            members.0
        };
        other_member
    }

    pub fn view(&self, current_user_id: String) -> Element<ChatMessage> {
        let other_member = Chat::get_other_member(current_user_id, &self.members);
        let nickname = text(other_member.clone());
        //let profile_picture = text(self.profile_picture.clone().unwrap_or(other_member));
        let pfp = Image::<image::Handle>::new("C:/Users/thisk/bot ava.png")
            .width(Length::Fixed(50.))
            .height(Length::Fixed(50.));

        let content = button(
            row![
                //profile_picture,
                pfp,
                column![nickname /* last_message */,] //.align_items(iced::Alignment::Start)
                                                      //.spacing(15)
            ]
            .width(Length::Fill)
            .spacing(10)
            .align_items(alignment::Alignment::Center),
        )
        .style(Button::Secondary)
        .on_press(ChatMessage::OpenChat)
        .width(Length::Fill);
        //.padding(10);
        container(content).width(Length::Fill).padding(5).into()
    }
}

#[derive(Clone)]
pub struct LetterList {
    client: reqwest::Client,
    session: Session,
    messages: HashMap<String, Message>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum LetterListMessage {
    Message(LetterMessage, String),
}
#[derive(Debug, Clone, PartialEq)]
pub enum LetterMessage {}
impl LetterList {
    pub fn update(&mut self, message: LetterListMessage) -> Command<LetterListMessage> {
        match message {
            LetterListMessage::Message(msg, id) => {
                match msg {
                    _ => {}
                }
                self.messages.get_mut(&id).unwrap().update(msg);
                Command::none()
            }
        }
    }
    pub fn view(&self, members: Vec<String>) -> Element<LetterListMessage> {
        let nickname_text = text(Chat::get_other_member(
            self.session.user_id.clone(),
            &members,
        ));
        column![
            row![nickname_text],
            scrollable(column(
                self.messages
                    .iter()
                    .map(|message| {
                        message
                            .1
                            .view()
                            .map(|msg| LetterListMessage::Message(msg, message.0.clone()))
                    })
                    .collect()
            )),
            row![]
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(5)
        .padding(5)
        .into()
    }
}

#[derive(Clone)]
pub struct Message {
    id: String,
    sender: String,
    content: String,
}

impl Message {
    pub fn update(&mut self, message: LetterMessage) -> Command<LetterMessage> {
        match message {
            _ => Command::none(),
        }
    }
    pub fn view(&self) -> Element<LetterMessage> {
        todo!()
    }
}
