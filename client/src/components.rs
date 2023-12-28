use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use iced::{
    alignment,
    widget::{
        button, column, container,
        image::{self, Handle},
        row, scrollable, text, Image,
    },
    Command, Element, Length,
};
use structs::requests::UserStatus;
use tokio::sync::Mutex;

use crate::{server_get, AppMessage};

pub struct ChatList {
    pub chats: HashMap<String, Chat>,
    pub client: reqwest::Client,
    pub profile_picture_cache: HashMap<String, String>,
}

impl ChatList {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            chats: HashMap::new(),
            client,
            profile_picture_cache: HashMap::new(),
        }
    }

    pub async fn get_profile_picture(&mut self, user: String) -> Option<String> {
        match self.profile_picture_cache.get(&user) {
            Some(pfp) => Some(pfp.clone()),
            None => {
                let pfp =
                    server_get::<UserStatus>(self.client.clone(), &format!("status/{user}"), None)
                        .await
                        .ok()
                        .and_then(|status| status.profile_picture)?;
                self.profile_picture_cache.insert(user, pfp.clone());
                Some(pfp)
            }
        }
    }

    pub fn view(&self, current_user_id: String) -> Element<ChatMessage> {
        scrollable(column(
            self.chats
                .iter()
                .map(|chat| chat.1.view(current_user_id.clone()))
                .collect(),
        ))
        .into()
    }
}

pub struct Chat {
    pub id: String,
    pub members: Vec<String>,
    pub messages: Vec<Message>,
    pub profile_picture: Option<String>,
}
pub struct Message {}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatMessage {
    OpenDirectMessage,
    ProfilePictureLoaded(Option<String>),
}

impl Chat {
    pub fn new(
        chat_list: Arc<Mutex<ChatList>>,
        current_user_id: String,
        id: String,
        members: Vec<String>,
    ) -> (Command<ChatMessage>, String) {
        chat_list.blocking_lock().chats.insert(
            id.clone(),
            Self {
                id: id.clone(),
                members: members.clone(),
                messages: vec![],
                profile_picture: None,
            },
        );

        let chat_list = chat_list.clone();
        (
            Command::perform(
                async move {
                    chat_list
                        .lock()
                        .await
                        .get_profile_picture(Chat::get_other_member(current_user_id, members))
                        .await
                },
                |pfp| ChatMessage::ProfilePictureLoaded(pfp),
            ),
            id,
        )
    }

    pub fn update(&mut self, message: ChatMessage) {
        match message {
            _ => todo!(),
            //ChatMessage::OpenDirectMessage => self.is_opened = true,
        }
    }

    pub fn get_other_member(current_user_id: String, members: Vec<String>) -> String {
        let members = match (members.get(0), members.get(1)) {
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
        let other_member = Chat::get_other_member(current_user_id, self.members.clone());
        let nickname = text(other_member.clone());
        let profile_picture = text(self.profile_picture.clone().unwrap_or(other_member));

        let content = button(
            row![
                profile_picture,
                column![nickname /* last_message */,]
                    //.align_items(iced::Alignment::Start)
                    .spacing(15)
            ]
            .spacing(10),
        );
        container(content).into()
    }
}

impl Message {
    pub fn view(&self, i: usize) -> Element<ChatMessage> {
        todo!()
    }
}
