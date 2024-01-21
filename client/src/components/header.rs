use iced::{
    widget::{container, text},
    Length,
};
use structs::requests::Session;

pub struct Header {
    pub session: Session,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeaderMessage {}

impl Header {
    pub fn update(&mut self, msg: HeaderMessage) -> iced::Command<HeaderMessage> {
        match msg {}
    }

    pub fn view(&self) -> iced::Element<HeaderMessage> {
        container(text("hello"))
            .center_y()
            .padding([0, 16])
            .width(Length::Fill)
            .height(Length::Fixed(50.0))
            .into()
    }
}
