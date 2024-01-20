pub use chrono::prelude::*;
pub use chrono::Duration;

pub mod requests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    pub struct Session {
        pub session_id: String,
        pub user_id: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct LoginInfo {
        pub username: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct UserStatus {
        pub id: String,
        pub display_name: String,
        pub profile_picture: Option<String>,
        pub online: bool,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateChat {
        pub other_members: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct LeaveChat {
        pub chat_id: String,
        pub other_members: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateMessage {
        pub chat_id: String,
        pub content: String,
        pub reply_to_id: Option<String>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct DeleteMessage {
        pub id: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct UpdateProfile {
        pub id: Option<String>,
        pub display_name: Option<String>,
        pub profile_picture: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    pub struct WsChatMessage {
        pub chat_id: String,
        pub sender_id: String,
        pub message_id: String,
        pub message: String,
        pub reply_to: Option<String>,
        pub created_at: super::DateTime<super::Utc>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
    pub struct WsCreateChat {
        pub chat_id: String,
        pub members: Vec<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
    pub struct WsLeaveChat {
        pub chat_id: String,
        pub member: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
    pub struct WsDeleteMessage {
        pub chat_id: String,
        pub message_id: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum WsMessageData {
        ChatMessage(WsChatMessage),
        CreateChat(WsCreateChat),
        LeaveChat(WsLeaveChat),
        DeleteMessage(WsDeleteMessage),
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct ChatWithMembers {
        pub id: String,
        pub members: Vec<String>,
        pub last_updated: super::DateTime<super::Utc>,
    }
}
