use iced::{
    widget::{container, image, row, text, Image},
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
        let pfp = Image::<image::Handle>::new("ava.png")
            .width(Length::Fixed(32.))
            .height(Length::Fixed(32.));
        container(
            row![pfp, text(self.session.user_id.clone()),]
                .align_items(iced::Alignment::Center)
                .spacing(8),
        )
        .center_y()
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fixed(50.0))
        .into()
    }
}
