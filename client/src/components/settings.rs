use iced::{
    theme::Button,
    widget::{button, column, container, radio, text, text_input},
    Length,
};
use structs::requests::{Session, UpdateProfile};

use crate::server::{get_profile_picture, server_post};

use super::{style_outline, ButtonStyle};

pub struct Settings {
    client: reqwest::Client,
    session: Session,
    loadtype: LoadType,
    profile_picture: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LoadType {
    URL,
    File,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMessage {
    ProfilePictureLoaded(Option<String>),
    ProfilePictureChanged(String),
    Error(String),
    ApplyChanges,
    ChangesApplied,
    ChangeLoadType(LoadType),
}

impl Settings {
    pub fn new(
        client: reqwest::Client,
        session: Session,
    ) -> (Self, iced::Command<SettingsMessage>) {
        let username = session.user_id.clone();
        (
            Self {
                client: client.clone(),
                session: session,
                loadtype: LoadType::URL,
                profile_picture: "".into(),
            },
            iced::Command::perform(
                get_profile_picture(client, username),
                SettingsMessage::ProfilePictureLoaded,
            ),
        )
    }

    pub fn update(&mut self, msg: SettingsMessage) -> iced::Command<SettingsMessage> {
        match msg {
            SettingsMessage::ProfilePictureLoaded(pfp) => {
                if let Some(pfp) = pfp {
                    self.profile_picture = pfp;
                }
                iced::Command::none()
            }
            SettingsMessage::ChangeLoadType(loadtype) => {
                self.loadtype = loadtype;
                iced::Command::none()
            }
            SettingsMessage::ProfilePictureChanged(profile_picture) => {
                self.profile_picture = profile_picture;
                iced::Command::none()
            }
            SettingsMessage::ApplyChanges => iced::Command::perform(
                server_post::<()>(
                    self.client.clone(),
                    "update_profile",
                    UpdateProfile {
                        profile_picture: if self.profile_picture.is_empty() {
                            None
                        } else {
                            Some(self.profile_picture.clone())
                        },
                    },
                    Some(self.session.session_id.clone()),
                ),
                |res| match res {
                    Ok(_) => SettingsMessage::ChangesApplied,
                    Err(err) => SettingsMessage::Error(err.to_string()),
                },
            ),
            SettingsMessage::ChangesApplied => iced::Command::none(),
            SettingsMessage::Error(_) => unreachable!(),
        }
    }

    pub fn view(&self) -> iced::Element<SettingsMessage> {
        let profile_picture_radio = [LoadType::URL, LoadType::File].iter().fold(
            column![text("Select load type:")],
            |column, loadtype| {
                column.push(radio(
                    format!("{loadtype:?}"),
                    *loadtype,
                    Some(self.loadtype),
                    SettingsMessage::ChangeLoadType,
                ))
            },
        );
        let profile_picture_input = match self.loadtype {
            LoadType::URL => container(
                text_input("Фото профиля", &self.profile_picture)
                    .on_input(SettingsMessage::ProfilePictureChanged)
                    .on_submit(SettingsMessage::ApplyChanges),
            ),
            //native-dialog
            LoadType::File => todo!(),
        };
        container(
            column![
                text("Настройки").size(28),
                profile_picture_radio,
                profile_picture_input,
                button("Сохранить")
                    .padding([8, 12])
                    .style(Button::Custom(Box::new(ButtonStyle::Blue)))
                    .on_press(SettingsMessage::ApplyChanges)
            ]
            .align_items(iced::Alignment::Center)
            .spacing(20),
        )
        .width(Length::Fixed(400.0))
        .padding(30)
        .style(style_outline)
        .into()
    }
}
