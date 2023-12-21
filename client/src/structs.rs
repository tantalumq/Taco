use iced::{
    alignment,
    widget::{
        button, column, container,
        image::{self, Handle},
        row, text, Image,
    },
    Command, Element, Length,
};

#[derive(Debug, Clone)]
pub enum UserStatus {
    ActiveUser(User),
    Guest(Guest),
}
#[derive(Debug, Clone)]
pub struct Guest {
    id: u64,
    pub page: GuestPage,
    pub input: InputData,
}
#[derive(Debug, Clone)]
pub struct InputData {
    pub linput: String,
    pub pinput: String,
    pub ninput: String,
    pub fncheck: bool,
    pub dcheck: bool,
    pub fninput: String,
    pub dinput: String,
}
#[derive(PartialEq, Debug, Clone)]
pub enum GuestPage {
    Register,
    Login,
}
#[derive(Debug, Clone)]
pub struct User {
    id: u64,
    nickname: String,
    full_name: Option<String>,
    description: Option<String>,
    status: Status,
    icon: Option<image::Handle>,
    friends: Vec<User>,
    direct_messages: Vec<DirectMessage>,
    pub message_input: String,
}
#[derive(Debug, Clone)]
pub struct DirectMessage {
    pub id: u64,
    pub participants: DirectMessageType,
    pub messages: Vec<UserMessage>,
    pub is_opened: bool,
}
#[derive(Debug, Clone)]
pub enum DirectMessageType {
    User((User, User)),
}
#[derive(Debug, Clone)]
pub enum AppDirectMessage {
    OpenDirectMessage,
}

#[derive(Debug, Clone)]
pub struct Group {
    id: u64,
    name: String,
    decription: String,
    participants: Vec<User>,
}
#[derive(Debug, Clone)]
pub enum UserMessage {
    Message(Message),
    ForwardedMessage(ForwardedMessage),
}
#[derive(Debug, Clone)]
pub struct Message {
    from: User,
    r#for: User,
    content: String,
    reply_for: Option<Box<UserMessage>>,
    //time: ?
}
#[derive(Debug, Clone)]
pub struct ForwardedMessage {
    from: User,
    r#for: User,
    content: Message,
    reply_for: Option<Box<UserMessage>>,
    //time: ?
}

#[derive(Debug, Clone)]
pub enum Status {
    Online(Platform),
    Offline(/* last time */),
}

#[derive(Debug, Clone)]
pub enum Platform {
    PC,
}
impl User {
    pub fn new(
        nickname: String,
        full_name: Option<String>,
        description: Option<String>,
        icon: Option<image::Handle>,
    ) -> User {
        User {
            id: 0,
            nickname,
            full_name,
            description,
            icon,
            status: Status::Offline(),
            friends: vec![],
            direct_messages: vec![
                DirectMessage {
                    id: 0,
                    participants: DirectMessageType::User((
                        User {
                            id: 1,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 1,
                    participants: DirectMessageType::User((
                        User {
                            id: 2,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 2,
                    participants: DirectMessageType::User((
                        User {
                            id: 4,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 3,
                    participants: DirectMessageType::User((
                        User {
                            id: 6,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 4,
                    participants: DirectMessageType::User((
                        User {
                            id: 8,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 5,
                    participants: DirectMessageType::User((
                        User {
                            id: 10,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 6,
                    participants: DirectMessageType::User((
                        User {
                            id: 12,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 7,
                    participants: DirectMessageType::User((
                        User {
                            id: 14,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 8,
                    participants: DirectMessageType::User((
                        User {
                            id: 16,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 9,
                    participants: DirectMessageType::User((
                        User {
                            id: 18,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 10,
                    participants: DirectMessageType::User((
                        User {
                            id: 20,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 11,
                    participants: DirectMessageType::User((
                        User {
                            id: 22,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 12,
                    participants: DirectMessageType::User((
                        User {
                            id: 24,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![],
                    is_opened: false,
                },
                DirectMessage {
                    id: 13,
                    participants: DirectMessageType::User((
                        User {
                            id: 26,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![UserMessage::Message(Message {
                        from: User {
                            id: 26,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        r#for: User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                        content: "toasdad".to_string(),
                        reply_for: None,
                    })],
                    is_opened: false,
                },
                DirectMessage {
                    id: 14,
                    participants: DirectMessageType::User((
                        User {
                            id: 28,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                    )),
                    messages: vec![UserMessage::Message(Message {
                        from: User {
                            id: 28,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas1".to_string(),
                            message_input: "".to_string(),
                        },
                        r#for: User {
                            id: 0,
                            description: None,
                            direct_messages: vec![],
                            friends: vec![],
                            full_name: None,
                            icon: None,
                            status: Status::Offline(),
                            nickname: "astas2".to_string(),
                            message_input: "".to_string(),
                        },
                        content: "toasdad".to_string(),
                        reply_for: None,
                    })],
                    is_opened: false,
                },
            ],
            message_input: "".to_string(),
        }
    }
    pub fn get(
        &self,
    ) -> (
        u64,
        &String,
        &Option<String>,
        &Option<String>,
        &Status,
        &Option<image::Handle>,
        &Vec<User>,
        &Vec<DirectMessage>,
    ) {
        (
            self.id,
            &self.nickname,
            &self.full_name,
            &self.description,
            &self.status,
            &self.icon,
            &self.friends,
            &self.direct_messages,
        )
    }
    pub fn get_mut(
        &mut self,
    ) -> (
        u64,
        &mut String,
        &mut Option<String>,
        &mut Option<String>,
        &mut Status,
        &mut Option<image::Handle>,
        &mut Vec<User>,
        &mut Vec<DirectMessage>,
    ) {
        (
            self.id,
            &mut self.nickname,
            &mut self.full_name,
            &mut self.description,
            &mut self.status,
            &mut self.icon,
            &mut self.friends,
            &mut self.direct_messages,
        )
    }
    pub fn current_user_load() -> UserStatus {
        UserStatus::Guest(Guest::new())
    }
}
impl Guest {
    fn new() -> Guest {
        Guest {
            id: 0,
            page: GuestPage::Register,
            input: InputData {
                linput: String::new(),
                pinput: String::new(),
                ninput: String::new(),
                fncheck: false,
                dcheck: false,
                fninput: String::new(),
                dinput: String::new(),
            },
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}
impl DirectMessage {
    fn new() -> DirectMessage {
        DirectMessage {
            id: 50,
            participants: DirectMessageType::User((
                User::new("asdsadasda".to_string(), None, None, None),
                User::new("asdsadasda".to_string(), None, None, None),
            )),
            messages: vec![],
            is_opened: false,
        }
    }
    pub fn update(&mut self, message: AppDirectMessage) {
        match message {
            AppDirectMessage::OpenDirectMessage => self.is_opened = true,
        }
    }
    pub fn view(&self, i: usize) -> Element<AppDirectMessage> {
        let direct_message_info = match &self.participants {
            DirectMessageType::User((userf, users)) => (
                Image::<Handle>::new("C:/Users/thisk/bot ava.png")
                    .height(50)
                    .width(50),
                format!(
                    "{} ({})",
                    users.nickname,
                    match users.full_name.to_owned() {
                        Some(str) => str,
                        None => String::from("Pablos"),
                    }
                ),
                match self.messages.last() {
                    Some(user_msg) => match user_msg {
                        UserMessage::Message(message) => {
                            format!("{}: {}", message.from.nickname, message.content)
                        }
                        UserMessage::ForwardedMessage(message) => {
                            format!("{}: {}", message.from.nickname, message.content.content)
                        }
                    },
                    None => "".to_string(),
                },
            ),
        };
        let nickname = text(direct_message_info.1);
        let last_message = text(direct_message_info.2);
        let button = container(
            button("")
                //.padding()
                .height(50)
                .width(30)
                .on_press(AppDirectMessage::OpenDirectMessage),
        )
        .align_y(alignment::Vertical::Center)
        .align_x(alignment::Horizontal::Right)
        .width(Length::Fill);
        let content = container(column![row![
            direct_message_info.0,
            column![nickname, last_message],
            button
        ]
        .width(Length::Fill)
        .padding(15)
        .spacing(10),]);
        container(content).width(500).into()
    }
}

impl UserMessage {
    pub fn view(&self, i: usize) -> Element<AppDirectMessage> {
        let message = match &self {
            UserMessage::Message(msg) => container(
                row![
                    column![
                        text(&msg.from.nickname).size(15),
                        text(&msg.content).size(15)
                    ],
                    Image::<Handle>::new("C:/Users/thisk/bot ava.png")
                        .height(35)
                        .width(35),
                ]
                .align_items(iced::Alignment::Center)
                .padding(15)
                .spacing(10),
            )
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right),
            UserMessage::ForwardedMessage(frwd_msg) => todo!(),
        };
        container(message).width(Length::Fill).into()
    }
}
impl Message {
    pub fn new(from: User, r#for: User, content: String) -> Message {
        Message {
            from,
            r#for,
            content,
            reply_for: None,
        }
    }
}
