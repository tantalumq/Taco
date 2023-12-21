use std::collections::HashMap;
use std::ops::Deref;

use iced::widget;
use iced::widget::text_input::Id;
use iced::widget::{
    button, checkbox, column, container, focus_next, row, scrollable, text, text_input,
};
use iced::{alignment, Application, Color, Command, Element, Executor, Length, Sandbox, Settings};

use once_cell::sync::Lazy;

static UDML_SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static UM_SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

static GRL_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static GRP_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static GRN_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static GRFN_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static GRD_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

static GLL_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static GLP_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

static AUM_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

mod db;
mod structs;
use db::{new_client, PrismaClient};

use structs::*;

#[tokio::main]
pub async fn main() -> iced::Result {
    Taco::run(Settings {
        window: iced::window::Settings {
            min_size: Some((600, 500)),
            position: iced::window::Position::Default,
            transparent: true,
            icon: None,
            resizable: false,
            ..Default::default()
        },
        ..Settings::with_flags(new_client().await.unwrap())
    })
}

struct Taco {
    user_status: UserStatus,
    client: PrismaClient,
}

#[derive(Debug, Clone)]
enum AppMessage {
    LoginInputChanged(String),
    NicknameInputChanged(String),
    FullNameCheckBoxChanged(bool),
    FullNameInputChanged(String),
    DescriptCheckBoxChanged(bool),
    DescriptionInputChanged(String),
    PasswordInputChanged(String),
    ContinueButtonPressed,
    ChangeFocus,
    DirectMessage((usize, AppDirectMessage)),
    UserMessage,
    UserMessageInputChanged(String),
    UserMessageSubmit(u64),
}

impl Application for Taco {
    type Message = AppMessage;

    type Executor = iced::executor::Default;

    type Theme = iced::theme::Theme;

    type Flags = PrismaClient;

    fn new(flags: PrismaClient) -> (Self, Command<AppMessage>) {
        (
            Taco {
                user_status: User::current_user_load(),
                client: flags,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Taco`s".try_into().unwrap()
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match &mut self.user_status {
            UserStatus::ActiveUser(user) => match message {
                AppMessage::ChangeFocus => focus_next::<AppMessage>(),
                AppMessage::DirectMessage((i, dm_message)) => {
                    if let Some(dm) = user.get_mut().7.get_mut(i) {
                        dm.update(dm_message);

                        Command::none()
                    } else {
                        Command::none()
                    }
                }
                AppMessage::UserMessage => Command::none(),
                AppMessage::UserMessageInputChanged(value) => {
                    user.message_input = value;

                    Command::none()
                }
                AppMessage::UserMessageSubmit(target_direct_id) => Command::none(),
                _ => Command::none(),
            },
            UserStatus::Guest(guest) => match message {
                AppMessage::ContinueButtonPressed => {
                    /* MAKE A REGISTER SYSTEM */
                    if guest.page != GuestPage::Login {
                        self.user_status = UserStatus::ActiveUser(User::new(
                            guest.input.ninput.to_owned(),
                            if guest.input.fncheck {
                                Some(guest.input.fninput.to_owned())
                            } else {
                                None
                            },
                            if guest.input.dcheck {
                                Some(guest.input.dinput.to_owned())
                            } else {
                                None
                            },
                            None,
                        ))
                    } else {
                        //UserStatus::ActiveUser(User::TODO)
                    }

                    Command::none()
                }
                AppMessage::LoginInputChanged(input) => {
                    guest.input.linput = input;

                    Command::none()
                }
                AppMessage::NicknameInputChanged(input) => {
                    guest.input.ninput = input;

                    Command::none()
                }
                AppMessage::FullNameCheckBoxChanged(value) => {
                    guest.input.fncheck = value;
                    Command::none()
                }
                AppMessage::FullNameInputChanged(input) => {
                    guest.input.fninput = input;

                    Command::none()
                }
                AppMessage::DescriptionInputChanged(input) => {
                    guest.input.dinput = input;

                    Command::none()
                }
                AppMessage::PasswordInputChanged(input) => {
                    guest.input.pinput = input;

                    Command::none()
                }
                AppMessage::ChangeFocus => focus_next::<AppMessage>(),
                AppMessage::DescriptCheckBoxChanged(value) => {
                    guest.input.dcheck = value;

                    Command::none()
                }
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.user_status {
            UserStatus::ActiveUser(user) => {
                /* MAKE A TACO */
                let direct_messages = container(
                    scrollable(column(
                        user.get()
                            .7
                            .iter()
                            .enumerate()
                            .map(|(i, dm)| {
                                dm.view(i)
                                    .map(move |message| AppMessage::DirectMessage((i, message)))
                            })
                            .collect(),
                    ))
                    .id(UDML_SCROLLABLE_ID.clone())
                    .width(500),
                )
                .align_x(alignment::Horizontal::Left);
                let mut um: Option<&Vec<UserMessage>> = None;
                let mut opened_direct: Option<&DirectMessage> = None;
                for dm in user.get().7 {
                    if dm.is_opened {
                        um = Some(&dm.messages);
                        opened_direct = Some(dm);
                    }
                }
                let user_messages = match um {
                    Some(message) => container(
                        scrollable(column(
                            message
                                .iter()
                                .enumerate()
                                .map(|(i, m)| m.view(i).map(|_| AppMessage::UserMessage))
                                .collect(),
                        ))
                        .id(UM_SCROLLABLE_ID.clone()),
                    )
                    .align_x(alignment::Horizontal::Center)
                    .width(Length::Fill)
                    .max_width(500),
                    None => container(
                        scrollable(text("Nothing. Выбери one Pablose"))
                            .id(UM_SCROLLABLE_ID.clone()),
                    )
                    .align_x(alignment::Horizontal::Center)
                    .width(Length::Fill)
                    .max_width(500),
                };
                let message_input = container(
                    text_input("Type message!", &user.message_input)
                        .id(AUM_TEXT_INPUT_ID.clone())
                        .on_input(AppMessage::UserMessageInputChanged)
                        .on_submit(AppMessage::UserMessageSubmit(match opened_direct {
                            Some(dm) => dm.id,
                            None => 0,
                        }))
                        .size(20),
                )
                .align_y(alignment::Vertical::Bottom)
                .align_x(alignment::Horizontal::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(15)
                .max_width(500);
                let content =
                    row![direct_messages, column![user_messages, message_input]].spacing(10);
                container(content).into()
            }
            UserStatus::Guest(guest) => {
                /* MAKE A REGISTER|LOGIN WINDOW */
                match guest.page {
                    GuestPage::Register => {
                        let title = text("Registration")
                            .size(50)
                            .style(Color::from([0.5, 0.5, 0.5]))
                            .width(Length::Fill)
                            .horizontal_alignment(alignment::Horizontal::Center);
                        let login_input = container(
                            text_input("Login", &guest.input.linput)
                                .id(GRL_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::LoginInputChanged)
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let nickname_input = container(
                            text_input("Nickname", &guest.input.ninput)
                                .id(GRN_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::NicknameInputChanged)
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let full_name_checkbox = container(checkbox(
                            "Full Name",
                            guest.input.fncheck,
                            AppMessage::FullNameCheckBoxChanged,
                        ))
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let description_checkbox = container(checkbox(
                            "Profile description",
                            guest.input.dcheck,
                            AppMessage::DescriptCheckBoxChanged,
                        ))
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let full_name_input = container(
                            text_input("Full name", &guest.input.fninput)
                                .id(GRFN_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::FullNameInputChanged)
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .center_x()
                        .align_x(alignment::Horizontal::Center);
                        let description_input = container(
                            text_input("Profile description", &guest.input.dinput)
                                .id(GRD_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::DescriptionInputChanged)
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .center_x()
                        .align_x(alignment::Horizontal::Center);
                        let password_input = container(
                            text_input("Password", &guest.input.pinput)
                                .id(GRP_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::PasswordInputChanged)
                                .password()
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let continue_button = container(
                            button("Continue")
                                .width(85)
                                .height(35)
                                .padding([5, 10])
                                .on_press(AppMessage::ContinueButtonPressed),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let row = match (guest.input.fncheck, guest.input.dcheck) {
                            (true, true) => row![full_name_input, description_input],
                            (true, false) => row![full_name_input],
                            (false, true) => row![description_input],
                            (false, false) => row![],
                        };
                        let content = column![
                            title,
                            login_input,
                            nickname_input,
                            row![full_name_checkbox, description_checkbox],
                            row,
                            password_input,
                            continue_button,
                        ]
                        .spacing(20)
                        .max_width(800);
                        container(content)
                            .width(Length::Fill)
                            .padding(40)
                            .center_x()
                            .into()
                    }
                    GuestPage::Login => {
                        let title = text("Login")
                            .size(50)
                            .style(Color::from([0.5, 0.5, 0.5]))
                            .width(Length::Fill)
                            .horizontal_alignment(alignment::Horizontal::Center);
                        let login_input = container(
                            text_input("Login", &guest.input.linput)
                                .id(GLL_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::LoginInputChanged)
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let password_input = container(
                            text_input("Password", &guest.input.pinput)
                                .id(GLP_TEXT_INPUT_ID.clone())
                                .padding(15)
                                .size(15)
                                .width(350)
                                .on_input(AppMessage::PasswordInputChanged)
                                .password()
                                .on_submit(AppMessage::ChangeFocus),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let continue_button = container(
                            button("Continue")
                                .width(85)
                                .height(35)
                                .padding([5, 10])
                                .on_press(AppMessage::ContinueButtonPressed),
                        )
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center);
                        let content = column![title, login_input, password_input, continue_button]
                            .spacing(20)
                            .max_width(800);
                        container(content)
                            .width(Length::Fill)
                            .padding(40)
                            .center_x()
                            .into()
                    }
                }
            }
        }
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
