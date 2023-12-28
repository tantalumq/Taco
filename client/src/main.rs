use std::{collections::HashMap, sync::Arc};

use components::{Chat, ChatList, ChatListMessage, ChatMessage};
use iced::{
    alignment,
    widget::{button, column, container, focus_next, row, scrollable, text, text_input},
    Application, Command, Element, Length, Settings,
};
use reqwest::header::HeaderMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use structs::requests::{ChatWithMembers, LoginInfo, Session, UserStatus};
use tokio::sync::Mutex;

const SERVER_URL: &'static str = "http://localhost:3000";

mod components;

#[tokio::main]
pub async fn main() -> iced::Result {
    let client = reqwest::Client::new();
    Taco::run(Settings {
        window: iced::window::Settings {
            min_size: Some((1, 1)),
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
}

enum AppState {
    LoggedIn {
        session: Session,
        chat_list: ChatList,
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
    ChatsLoaded(Vec<ChatWithMembers>),
    ChatListMessage(ChatListMessage),
}

#[derive(Debug)]
enum ServerRequestError {
    ReqwestError(reqwest::Error),
    InvalidDataError(serde_json::Error),
    InvalidResponseError(serde_json::Error),
    Status(reqwest::Error),
}

async fn server_post<T: DeserializeOwned>(
    client: reqwest::Client,
    route: &'static str,
    data: impl Serialize,
    session: Option<String>,
) -> Result<T, ServerRequestError> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    if let Some(session) = session {
        headers.insert(
            "Authorization",
            format!("Bearer {session}").parse().unwrap(),
        );
    }

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
        .map_err(ServerRequestError::ReqwestError)?
        .error_for_status()
        .map_err(ServerRequestError::Status)?;
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
    session: Option<String>,
) -> Result<T, ServerRequestError> {
    let mut headers = HeaderMap::new();

    if let Some(session) = session {
        headers.insert(
            "Authorization",
            format!("Bearer {session}").parse().unwrap(),
        );
    }

    let response = client
        .get(&format!("{SERVER_URL}/{route}"))
        .headers(headers)
        .send()
        .await
        .map_err(ServerRequestError::ReqwestError)?
        .error_for_status()
        .map_err(ServerRequestError::Status)?;
    let response_data = response
        .text()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_value =
        serde_json::from_str(&response_data).map_err(ServerRequestError::InvalidDataError)?;
    Ok(response_value)
}

pub async fn get_profile_picture(client: reqwest::Client, user: String) -> Option<String> {
    server_get::<UserStatus>(client, &format!("status/{user}"), None)
        .await
        .ok()
        .and_then(|status| status.profile_picture)
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
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Taco`s".try_into().unwrap()
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match self.state {
            AppState::LoggedIn {
                ref session,
                ref mut chat_list,
            } => match message {
                AppMessage::ChatListMessage(msg) => chat_list
                    .update(msg)
                    .map(|msg| AppMessage::ChatListMessage(msg)),
                AppMessage::ChatsLoaded(loaded_chats) => {
                    let chats: Vec<(Command<ChatMessage>, String)> = loaded_chats
                        .into_iter()
                        .map(|chat| {
                            Chat::new(chat_list, session.clone().user_id, chat.id, chat.members)
                        })
                        .collect();
                    Command::batch(chats.into_iter().map(|(cmd, chat_id)| {
                        cmd.map(move |msg| {
                            AppMessage::ChatListMessage(ChatListMessage::ChatMessage(
                                msg,
                                chat_id.clone(),
                            ))
                        })
                    }))
                }
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
                        None,
                    ),
                    move |register_result| AppMessage::LoggedIn(register_result.unwrap()),
                ),
                AppMessage::LoggedIn(session) => {
                    self.state = AppState::LoggedIn {
                        session: session.clone(),
                        chat_list: ChatList::new(self.client.clone(), session.clone()),
                    };
                    Command::perform(
                        server_get::<Vec<ChatWithMembers>>(
                            self.client.clone(),
                            "chats",
                            Some(session.session_id),
                        ),
                        move |chats| AppMessage::ChatsLoaded(chats.unwrap()),
                    )
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.state {
            AppState::LoggedIn {
                ref session,
                ref chat_list,
            } => {
                let username = &session.user_id;
                let chats = chat_list
                    .view(username.clone())
                    .map(AppMessage::ChatListMessage);
                column![chats]
                    .max_width(350)
                    .padding(15)
                    .align_items(iced::Alignment::Start)
                    .into()
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
                    .width(Length::Fixed(300.))
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
