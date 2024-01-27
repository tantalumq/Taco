use iced::{
    theme::Button,
    widget::{button, column, container, row, text, text_input},
    Length,
};
use native_dialog::FileDialog;

use reqwest::{multipart, Body};
use structs::requests::{Session, UpdateProfile};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::server::{self, get_profile_picture, server_post};

use super::{style_outline, ButtonStyle};

pub struct Settings {
    client: reqwest::Client,
    session: Session,
    profile_picture: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMessage {
    ProfilePictureSelecting,
    ProfilePictureLoaded(Option<String>),
    ProfilePictureChanged(String),
    Error(String),
    ApplyChanges,
    ChangesApplied,
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
            SettingsMessage::ProfilePictureSelecting => {
                let path = FileDialog::new()
                    .set_location("~/Downloads")
                    .add_filter("PNG Image", &["png"])
                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                    .show_open_single_file()
                    .unwrap();

                let path = match path {
                    Some(path) => path,
                    None => return iced::Command::none(),
                };
                let client = self.client.clone();
                iced::Command::perform(
                    async move {
                        let file = tokio::fs::File::open(&path).await.unwrap();
                        let stream = FramedRead::new(file, BytesCodec::new());
                        let file_body = Body::wrap_stream(stream);
                        let format = path.extension().unwrap().to_str().unwrap();
                        let part = multipart::Part::stream(file_body)
                            .mime_str(&format!("image/{format}"))
                            .unwrap();
                        let form = multipart::Form::new().part("file", part);
                        client
                            .post(format!("{}/upload_picture", server::SERVER_URL))
                            .multipart(form)
                            .send()
                            .await
                            .unwrap()
                            .text()
                            .await
                            .ok()
                            .map(|id| format!("{}/content/img-{}", server::SERVER_URL, &id))
                    },
                    SettingsMessage::ProfilePictureLoaded,
                )
            }
            SettingsMessage::ProfilePictureLoaded(pfp) => {
                if let Some(pfp) = pfp {
                    self.profile_picture = pfp;
                }
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
            SettingsMessage::ProfilePictureChanged(pfp) => {
                self.profile_picture = pfp;
                iced::Command::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<SettingsMessage> {
        container(
            column![
                text("Настройки").size(28),
                row![
                    text_input("Ссылка на фото", &self.profile_picture)
                        .on_input(SettingsMessage::ProfilePictureChanged)
                        .on_submit(SettingsMessage::ApplyChanges),
                    button("Загрузить фото")
                        .on_press(SettingsMessage::ProfilePictureSelecting)
                        .style(Button::Custom(Box::new(ButtonStyle::Blue)))
                ]
                .width(Length::Fill)
                .padding(10)
                .spacing(10),
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
