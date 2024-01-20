use iced::{
    widget::{container, text},
    Length,
};
use structs::requests::Session;

pub struct Header {
    session: Session,
    profile_picture: Option<String>,
}

pub enum HeaderMessage {}

impl Header {
    fn update(&mut self, msg: HeaderMessage) -> iced::Command<HeaderMessage> {
        match msg {}
    }

    fn view(&self) -> iced::Element<HeaderMessage> {
        container(text("hello"))
            .width(Length::Fill)
            .height(Length::Fixed(20.0))
            .into()
    }
}
