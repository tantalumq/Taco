use crate::server::server_post;
use iced::{
    alignment,
    theme::Button,
    widget::{button, column, container, row, text, text_input},
    Length,
};
use structs::requests::{LoginInfo, Session};

use super::ButtonStyle;

#[derive(Default)]
pub struct LoginScreen {
    pub logging_in: bool,
    username_input: String,
    password_input: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginScreenMessage {
    UsernameInputChanged(String),
    PasswordInputChanged(String),
    Login,
    Register,
    LoggedIn(Session),
    FocusChange,
    Error(String),
}

impl LoginScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: LoginScreenMessage) -> iced::Command<LoginScreenMessage> {
        match message {
            LoginScreenMessage::UsernameInputChanged(value) => {
                self.username_input = value;
                iced::Command::none()
            }
            LoginScreenMessage::PasswordInputChanged(value) => {
                self.password_input = value;
                iced::Command::none()
            }
            LoginScreenMessage::FocusChange => iced::widget::focus_next(),
            LoginScreenMessage::Register | LoginScreenMessage::Login => {
                self.logging_in = true;
                iced::Command::perform(
                    server_post::<Session>(
                        self.client.clone(),
                        if message == LoginScreenMessage::Login {
                            "login"
                        } else {
                            "register"
                        },
                        LoginInfo {
                            username: self.username_input.clone(),
                            password: self.password_input.clone(),
                        },
                        None,
                    ),
                    move |register_result| match register_result {
                        Ok(session) => LoginScreenMessage::LoggedIn(session),
                        Err(err) => LoginScreenMessage::Error(err.to_string()),
                    },
                )
            }
            LoginScreenMessage::LoggedIn(_) => unreachable!(),
            LoginScreenMessage::Error(_) => unreachable!(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<LoginScreenMessage> {
        iced::Subscription::none()
    }

    pub fn view(&self) -> iced::Element<LoginScreenMessage> {
        let sign_up_text = text("Вход").size(36);
        let username_text_input = text_input("Имя пользователя", &self.username_input)
            .on_input(LoginScreenMessage::UsernameInputChanged)
            .on_submit(LoginScreenMessage::FocusChange)
            .padding(10);
        let password_text_input = text_input("Пароль", &self.password_input)
            .on_input(LoginScreenMessage::PasswordInputChanged)
            .on_submit(LoginScreenMessage::FocusChange)
            .password()
            .padding(10);

        let center_button = |content| {
            button(text(content).horizontal_alignment(alignment::Horizontal::Center))
                .width(Length::Fill)
                .style(Button::Custom(Box::new(ButtonStyle::Blue)))
                .padding(10)
        };

        let button_row = if self.logging_in {
            row![center_button("Войти"), center_button("Зарегистрироваться"),]
        } else {
            row![
                center_button("Войти").on_press(LoginScreenMessage::Login),
                center_button("Зарегистрироваться").on_press(LoginScreenMessage::Register),
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
}
