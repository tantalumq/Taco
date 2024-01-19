mod components;
use std::fmt::Display;

use components::{
    chat::{Chat, ChatMessage},
    chat_list::{ChatList, ChatListMessage},
    ChatButtonStyle,
};
use iced::{
    alignment, font,
    theme::Button,
    widget::{button, column, container, focus_next, row, text, text_input},
    Application, Color, Command, Element, Font, Length, Settings,
};
use iced_aw::modal;
use reqwest::{header::HeaderMap, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use structs::requests::{ChatWithMembers, LoginInfo, Session, UserStatus};

mod ws_client;

const SERVER_URL: &'static str = "http://localhost:3000";

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
        default_font: Font::with_name("Inter"),
        ..Settings::with_flags(client)
    })
}

struct Taco {
    state: AppState,
    client: reqwest::Client,
    error: Option<String>,
}

enum AppState {
    LoggedIn {
        session: Session,
        chat_list: ChatList,
    },
    Guest {
        username_input: String,
        password_input: String,
        logging_in: bool,
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
    FontLoaded(Result<(), font::Error>),
    Error(String),
    CloseError,
    WsEvent(ws_client::WsEvent),
}

#[derive(Debug)]
enum ServerRequestError {
    ReqwestError(reqwest::Error),
    InvalidDataError(serde_json::Error),
    InvalidResponseError(serde_json::Error),
    Status(StatusCode, Option<String>),
}

impl Display for ServerRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerRequestError::ReqwestError(err) => err.fmt(f),
            ServerRequestError::InvalidDataError(err) => err.fmt(f),
            ServerRequestError::InvalidResponseError(err) => err.fmt(f),
            ServerRequestError::Status(status, msg) => {
                write!(
                    f,
                    "Status Code {:#?}: {}",
                    status,
                    msg.as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("Неизвестная ошибка.")
                )
            }
        }
    }
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
        .map_err(ServerRequestError::ReqwestError)?;
    let response = match response.status() {
        StatusCode::OK => Ok(response),
        status => Err(ServerRequestError::Status(
            status,
            response.text().await.ok(),
        )),
    }?;
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
    route: String,
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
        .map_err(ServerRequestError::ReqwestError)?;
    let response = match response.status() {
        StatusCode::OK => Ok(response),
        status => Err(ServerRequestError::Status(
            status,
            response.text().await.ok(),
        )),
    }?;
    let response_data = response
        .text()
        .await
        .map_err(ServerRequestError::ReqwestError)?;
    let response_value =
        serde_json::from_str(&response_data).map_err(ServerRequestError::InvalidDataError)?;
    Ok(response_value)
}

pub async fn get_profile_picture(client: reqwest::Client, user: String) -> Option<String> {
    server_get::<UserStatus>(client, format!("status/{user}"), None)
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
                    logging_in: false,
                },
                client,
                error: None,
            },
            font::load(include_bytes!("../fonts/inter.ttf").as_slice()).map(AppMessage::FontLoaded),
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
                AppMessage::ChatListMessage(msg) => {
                    if let ChatListMessage::Error(err) = msg {
                        self.update(AppMessage::Error(err))
                    } else {
                        chat_list
                            .update(msg)
                            .map(|msg| AppMessage::ChatListMessage(msg))
                    }
                }
                AppMessage::ChatsLoaded(loaded_chats) => {
                    let chats: Vec<(Command<ChatMessage>, String)> = loaded_chats
                        .into_iter()
                        .map(|chat| {
                            Chat::new(
                                chat_list,
                                session.clone().user_id,
                                chat.id,
                                chat.members,
                                chat.last_updated,
                            )
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
                AppMessage::Error(err) => {
                    self.error = Some(err);
                    Command::none()
                }
                AppMessage::CloseError => {
                    self.error = None;
                    Command::none()
                }
                _ => Command::none(),
            },

            AppState::Guest {
                ref mut username_input,
                ref mut password_input,
                ref mut logging_in,
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
                AppMessage::Register | AppMessage::Login => {
                    *logging_in = true;
                    Command::perform(
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
                        move |register_result| match register_result {
                            Ok(session) => AppMessage::LoggedIn(session),
                            Err(err) => AppMessage::Error(err.to_string()),
                        },
                    )
                }
                AppMessage::LoggedIn(session) => {
                    self.state = AppState::LoggedIn {
                        session: session.clone(),
                        chat_list: ChatList::new(self.client.clone(), session.clone()),
                    };
                    Command::perform(
                        server_get::<Vec<ChatWithMembers>>(
                            self.client.clone(),
                            "chats".into(),
                            Some(session.session_id),
                        ),
                        move |chats| AppMessage::ChatsLoaded(chats.unwrap()),
                    )
                }
                AppMessage::Error(err) => {
                    self.error = Some(err);
                    *logging_in = false;
                    Command::none()
                }
                AppMessage::CloseError => {
                    self.error = None;
                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let overlay = self.error.as_ref().map(|err| {
            container(
                text(err)
                    .style(Color::from_rgba8(255, 0, 0, 1.0))
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .padding([0, 50])
        });

        modal(
            match self.state {
                AppState::LoggedIn {
                    ref session,
                    ref chat_list,
                } => {
                    let username = &session.user_id;
                    let chats = container(
                        chat_list
                            .view(username.clone())
                            .map(AppMessage::ChatListMessage),
                    )
                    .width(Length::Fill)
                    .padding(10);
                    chats
                }
                AppState::Guest {
                    ref username_input,
                    ref password_input,
                    logging_in,
                } => {
                    let sign_up_text = text("Вход").size(36);
                    let username_text_input = text_input("Имя пользователя", username_input)
                        .on_input(AppMessage::UsernameInputChanged)
                        .on_submit(AppMessage::FocusChange)
                        .padding(10);
                    let password_text_input = text_input("Пароль", password_input)
                        .on_input(AppMessage::PasswordInputChanged)
                        .on_submit(AppMessage::FocusChange)
                        .password()
                        .padding(10);

                    let center_button = |content| {
                        button(
                            text(content)
                                //.size(13)
                                .horizontal_alignment(alignment::Horizontal::Center),
                        )
                        .width(Length::Fill)
                        .style(Button::Custom(Box::new(ChatButtonStyle::SenderMessage)))
                        .padding(10)
                    };

                    let button_row = if logging_in {
                        row![center_button("Войти"), center_button("Зарегистрироваться"),]
                    } else {
                        row![
                            center_button("Войти").on_press(AppMessage::Login),
                            center_button("Зарегистрироваться").on_press(AppMessage::Register),
                        ]
                    }
                    .spacing(10);
                    container(
                        column![
                            sign_up_text,
                            username_text_input,
                            password_text_input,
                            button_row
                        ]
                        .spacing(10)
                        .width(Length::Fixed(400.))
                        .align_items(iced::Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
                }
            },
            overlay,
        )
        .backdrop(AppMessage::CloseError)
        .on_esc(AppMessage::CloseError)
        .align_y(alignment::Vertical::Center)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::default()
    }

    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        <Self::Theme as iced::application::StyleSheet>::Style::default()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match &self.state {
            AppState::LoggedIn { session, .. } => ws_client::connect(session.session_id.clone())
                .map(|event| AppMessage::WsEvent(event)),
            AppState::Guest { .. } => iced::Subscription::none(),
        }
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }
}
