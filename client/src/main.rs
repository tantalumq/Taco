use std::collections::HashMap;

use iced::{
    alignment,
    widget::{button, column, container, focus_next, row, text, text_input},
    Application, Command, Element, Length, Settings,
};
use reqwest::header::HeaderMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use structs::requests::{LoginInfo, Session, UserStatus};

const SERVER_URL: &'static str = "http://localhost:3000";

mod components;

#[tokio::main]
pub async fn main() -> iced::Result {
    let client = reqwest::Client::new();
    Taco::run(Settings {
        window: iced::window::Settings {
            min_size: Some((600, 500)),
            position: iced::window::Position::Default,
            transparent: true,
            icon: None,
            ..Default::default()
        },
        ..Settings::with_flags(client)
    })
}

struct Taco {
    state: AppState,
    client: reqwest::Client,
    profile_picture_cache: HashMap<String, String>,
}

enum AppState {
    LoggedIn {
        session: Session,
    },
    Guest {
        username_input: String,
        password_input: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum AppMessage {
    UsernameInputChanged(String),
    PasswordInputChanged(String),
    Login,
    Register,
    LoggedIn(Session),
    FocusChange,
}

#[derive(Debug)]
enum ServerRequestError {
    ReqwestError(reqwest::Error),
    InvalidDataError(serde_json::Error),
    InvalidResponseError(serde_json::Error),
}

async fn server_post<T: DeserializeOwned>(
    client: reqwest::Client,
    route: &'static str,
    data: impl Serialize,
) -> Result<T, ServerRequestError> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let response = client
        .post(&format!("{SERVER_URL}/{route}"))
        .headers(headers)
        .body(
            serde_json::to_value(data)
                .map_err(ServerRequestError::InvalidResponseError)?
                .to_string(),
        )
        .send()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_data = response
        .text()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_value =
        serde_json::from_str(&response_data).map_err(ServerRequestError::InvalidDataError)?;
    Ok(response_value)
}

async fn server_get<T: DeserializeOwned>(
    client: reqwest::Client,
    route: &str,
) -> Result<T, ServerRequestError> {
    let response = client
        .get(&format!("{SERVER_URL}/{route}"))
        .send()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_data = response
        .text()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_value =
        serde_json::from_str(&response_data).map_err(ServerRequestError::InvalidDataError)?;
    Ok(response_value)
}

impl Taco {
    async fn get_profile_picture(&mut self, user: String) -> Option<String> {
        match self.profile_picture_cache.get(&user) {
            Some(pfp) => Some(pfp.clone()),
            None => {
                let pfp = server_get::<UserStatus>(self.client.clone(), &format!("status/{user}"))
                    .await
                    .ok()
                    .and_then(|status| status.profile_picture)?;
                self.profile_picture_cache.insert(user, pfp.clone());
                Some(pfp)
            }
        }
    }
}

impl Application for Taco {
    type Message = AppMessage;

    type Executor = iced::executor::Default;

    type Theme = iced::theme::Theme;

    type Flags = reqwest::Client;

    fn new(client: reqwest::Client) -> (Self, Command<AppMessage>) {
        (
            Taco {
                state: AppState::Guest {
                    username_input: String::new(),
                    password_input: String::new(),
                },
                client,
                profile_picture_cache: HashMap::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Taco`s".try_into().unwrap()
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match self.state {
            AppState::LoggedIn { ref session } => match message {
                _ => Command::none(),
            },
            AppState::Guest {
                ref mut username_input,
                ref mut password_input,
            } => match message {
                AppMessage::UsernameInputChanged(value) => {
                    *username_input = value;
                    Command::none()
                }
                AppMessage::PasswordInputChanged(value) => {
                    *password_input = value;
                    Command::none()
                }
                AppMessage::FocusChange => focus_next(),
                AppMessage::Register | AppMessage::Login => Command::perform(
                    server_post::<Session>(
                        self.client.clone(),
                        if message == AppMessage::Login {
                            "login"
                        } else {
                            "register"
                        },
                        LoginInfo {
                            username: username_input.clone(),
                            password: password_input.clone(),
                        },
                    ),
                    move |register_result| AppMessage::LoggedIn(register_result.unwrap()),
                ),
                AppMessage::LoggedIn(session) => {
                    self.state = AppState::LoggedIn { session };
                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.state {
            AppState::LoggedIn { ref session } => {
                let username = &session.user_id;
                text(format!("Welcome, {username}!")).into()
            }
            AppState::Guest {
                ref username_input,
                ref password_input,
            } => {
                let sign_up_text = text("Sign Up").size(36);
                let username_text_input = text_input("Username", username_input)
                    .on_input(AppMessage::UsernameInputChanged)
                    .on_submit(AppMessage::FocusChange)
                    .padding(10);
                let password_text_input = text_input("Password", password_input)
                    .on_input(AppMessage::PasswordInputChanged)
                    .on_submit(AppMessage::FocusChange)
                    .password()
                    .padding(10);

                let center_button = |content| {
                    button(text(content).horizontal_alignment(alignment::Horizontal::Center))
                        .width(Length::Fill)
                        .padding(10)
                };

                let button_row = row![
                    center_button("Log in").on_press(AppMessage::Login),
                    center_button("Register").on_press(AppMessage::Register),
                ]
                .spacing(10);
                container(
                    column![
                        sign_up_text,
                        username_text_input,
                        password_text_input,
                        button_row
                    ]
                    .spacing(10)
                    .width(Length::Fixed(250.))
                    .align_items(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
            }
        }

        //container(content).into()
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::default()
    }

    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        <Self::Theme as iced::application::StyleSheet>::Style::default()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::none()
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}
