use iced::{
    alignment, font,
    widget::{container, text},
    Application, Color, Command, Element, Font, Length, Settings,
};
use iced_aw::modal;

use components::{
    header::HeaderMessage,
    login_screen::{LoginScreen, LoginScreenMessage},
    main_screen::{MainScreen, MainScreenMessage},
};
use server::server_post;

mod components;
mod server;
mod ws_client;

#[tokio::main]
pub async fn main() -> iced::Result {
    let client = reqwest::Client::new();
    Taco::run(Settings {
        window: iced::window::Settings {
            min_size: Some((320, 240)),
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
    LoggedIn(MainScreen),
    Guest(LoginScreen),
}

#[derive(Debug, Clone, PartialEq)]
enum AppMessage {
    LoginScreen(LoginScreenMessage),
    MainScreen(MainScreenMessage),
    FontsLoaded(Result<(), font::Error>),
    Error(String),
    CloseError,
    LoggedOut,
}

impl Application for Taco {
    type Executor = iced::executor::Default;

    type Message = AppMessage;

    type Theme = iced::theme::Theme;

    type Flags = reqwest::Client;

    fn new(client: reqwest::Client) -> (Self, Command<AppMessage>) {
        (
            Taco {
                state: AppState::Guest(LoginScreen::new()),
                client,
                error: None,
            },
            Command::batch(vec![
                font::load(include_bytes!("../fonts/inter.ttf").as_slice()),
                font::load(include_bytes!("../fonts/icons.ttf").as_slice()),
            ])
            .map(AppMessage::FontsLoaded),
        )
    }

    fn title(&self) -> String {
        "Taco`s".try_into().unwrap()
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match message {
            AppMessage::Error(err) => {
                self.error = Some(err);
                if let AppState::Guest(ref mut login_screen) = self.state {
                    login_screen.logging_in = false;
                }
                Command::none()
            }
            AppMessage::CloseError => {
                self.error = None;
                Command::none()
            }
            AppMessage::LoggedOut => {
                self.state = AppState::Guest(LoginScreen::new());
                Command::none()
            }
            _ => match self.state {
                AppState::LoggedIn(ref mut main_screen) => {
                    if let AppMessage::MainScreen(msg) = message {
                        match msg {
                            MainScreenMessage::Header(HeaderMessage::LogOut) => Command::perform(
                                server_post::<()>(
                                    self.client.clone(),
                                    "logout",
                                    (),
                                    Some(main_screen.session.session_id.clone()),
                                ),
                                |_| AppMessage::LoggedOut,
                            ),
                            _ => main_screen.update(msg).map(|msg| {
                                if let MainScreenMessage::Error(err) = msg {
                                    AppMessage::Error(err)
                                } else {
                                    AppMessage::MainScreen(msg)
                                }
                            }),
                        }
                    } else {
                        Command::none()
                    }
                }
                AppState::Guest(ref mut login_screen) => {
                    if let AppMessage::LoginScreen(msg) = message {
                        match msg {
                            LoginScreenMessage::LoggedIn(session) => {
                                let (screen, cmd) = MainScreen::new(session, self.client.clone());
                                self.state = AppState::LoggedIn(screen);
                                cmd.map(AppMessage::MainScreen)
                            }
                            _ => login_screen.update(msg).map(|msg| {
                                if let LoginScreenMessage::Error(err) = msg {
                                    AppMessage::Error(err)
                                } else {
                                    AppMessage::LoginScreen(msg)
                                }
                            }),
                        }
                    } else {
                        Command::none()
                    }
                }
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
                AppState::LoggedIn(ref main_screen) => {
                    main_screen.view().map(AppMessage::MainScreen)
                }
                AppState::Guest(ref login_screen) => {
                    login_screen.view().map(AppMessage::LoginScreen)
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
            AppState::LoggedIn(main_screen) => {
                main_screen.subscription().map(AppMessage::MainScreen)
            }
            AppState::Guest(login_screen) => {
                login_screen.subscription().map(AppMessage::LoginScreen)
            }
        }
    }

    fn scale_factor(&self) -> f64 {
        1.125
    }
}
