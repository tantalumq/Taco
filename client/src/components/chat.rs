use super::{chat_list::ChatList, ChatButtonStyle};
use crate::server::get_profile_picture;
use iced::{
    alignment,
    theme::Button,
    widget::{button, column, container, image, row, text, Image},
    Command, Element, Length,
};
use structs::{DateTime, Utc};

#[derive(Clone)]
pub struct Chat {
    pub id: String,
    pub members: Vec<String>,
    pub profile_picture: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub is_open: bool,
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
        last_updated: DateTime<Utc>,
    ) -> (Command<ChatMessage>, String) {
        chat_list.chats.insert(
            id.clone(),
            Self {
                id: id.clone(),
                members: members.clone(),
                profile_picture: None,
                is_open: false,
                last_updated,
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
        }
    }

    pub fn get_other_member(current_user_id: String, members: &[String]) -> String {
        let first_member = members.get(0);
        let members = match (first_member, members.get(1).or(first_member)) {
            (Some(a), Some(b)) => (a.clone(), b.clone()),
            _ => panic!("group chats are not supported ( иди нахуй шаман )"),
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

        let chat_button_style = if self.is_open {
            ChatButtonStyle::Open
        } else {
            ChatButtonStyle::Closed
        };

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
        .style(Button::Custom(Box::new(chat_button_style)))
        .on_press(ChatMessage::OpenChat)
        .width(Length::Fill);
        //.padding(10);
        container(content).width(Length::Fill).padding(5).into()
    }
}
