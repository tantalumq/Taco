use iced::{
    widget::{container, row, text, Space},
    Color, Length, Theme,
};
use structs::requests::Session;

use crate::server::get_profile_picture;

use super::{
    icon_button,
    web_image::{WebImage, WebImageMessage},
};

pub struct Header {
    pub session: Session,
    pub profile_picture: WebImage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeaderMessage {
    ProfilePicture(WebImageMessage),
    ProfilePictureLoaded(Option<String>),
    SettingsOpen,
    LogOut,
}

impl Header {
    pub fn new(session: Session, client: reqwest::Client) -> (Self, iced::Command<HeaderMessage>) {
        let user = session.user_id.clone();
        let header = Self {
            session,
            profile_picture: WebImage::new(client.clone()),
        };
        (
            header,
            iced::Command::perform(
                get_profile_picture(client, user),
                HeaderMessage::ProfilePictureLoaded,
            ),
        )
    }

    pub fn update(&mut self, msg: HeaderMessage) -> iced::Command<HeaderMessage> {
        match msg {
            HeaderMessage::ProfilePicture(msg) => self
                .profile_picture
                .update(msg)
                .map(HeaderMessage::ProfilePicture),
            HeaderMessage::ProfilePictureLoaded(pfp) => {
                if let Some(pfp) = pfp {
                    self.profile_picture
                        .load_image(pfp)
                        .map(HeaderMessage::ProfilePicture)
                } else {
                    iced::Command::none()
                }
            }
            HeaderMessage::SettingsOpen | HeaderMessage::LogOut => unreachable!(),
        }
    }

    pub fn view(&self) -> iced::Element<HeaderMessage> {
        let pfp = self
            .profile_picture
            .view()
            .width(Length::Fixed(50.))
            .height(Length::Fixed(50.));
        let container_style = |_: &Theme| container::Appearance {
            border_radius: 0.0.into(),
            border_width: 1.0,
            border_color: Color::from_rgba8(0, 0, 0, 0.4),
            ..Default::default()
        };
        container(
            row![
                pfp,
                text(self.session.user_id.clone()),
                Space::with_width(Length::Fill),
                icon_button('').on_press(HeaderMessage::SettingsOpen),
                icon_button('').on_press(HeaderMessage::LogOut)
            ]
            .align_items(iced::Alignment::Center)
            .spacing(8),
        )
        .style(container_style)
        .center_y()
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fixed(64.0))
        .into()
    }
}
