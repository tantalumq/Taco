use std::{cell::RefCell, rc::Rc};

use iced::{
    alignment,
    widget::{
        button, column, container,
        image::{self, Handle},
        row, text, Image,
    },
    Command, Element, Length,
};

use crate::Taco;

pub struct Chat {
    pub id: String,
    pub members: Vec<String>,
    pub messages: Vec<Message>,
    pub taco: Rc<RefCell<Taco>>,
}
pub struct Message {}

#[derive(Debug, Clone)]
enum ChatMessage {
    OpenDirectMessage,
    ProfilePictureLoaded(String),
}

impl Chat {
    pub fn new(
        id: String,
        members: Vec<String>,
        messages: Vec<Message>,
        taco: Rc<RefCell<Taco>>,
    ) -> (Command, Self) {
        Self {
            id,
            members,
            messages,
            taco,
        }
    }

    pub fn update(&mut self, message: ChatMessage) {
        match message {
            _ => todo!(),
            //ChatMessage::OpenDirectMessage => self.is_opened = true,
        }
    }

    pub fn view(&self, current_user_id: String) -> Element<ChatMessage> {
        let members = match (self.members.get(0), self.members.get(1)) {
            (Some(a), Some(b)) => (a.clone(), b.clone()),
            _ => panic!("group chats are not supported"),
        };

        let other_member = if members.0 == current_user_id {
            members.1
        } else {
            members.0
        };
        let nickname = text(other_member.clone());
        let profile_picture = self.taco.get_mut().get_profile_picture(other_member);

        // let last_message = ;

        let content = button(
            row![
                /* profile_picture, */
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
