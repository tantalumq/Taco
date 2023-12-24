pub mod requests {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct UserQuery {
        pub id: String,
    }

    #[derive(Clone, Deserialize, Serialize)]
    pub struct Session {
        pub session_id: String,
        pub user_id: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct LoginInfo {
        pub username: String,
        pub password: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct CreateChat {
        pub other_members: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct LeaveChat {
        pub chat_id: String,
        pub other_members: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct GetChatMessages {
        pub chat_id: String,
    }
    #[derive(Deserialize, Serialize)]
    pub struct CreateMessage {
        pub chat_id: String,
        pub message_id: String,
        pub content: String,
        pub reply_to_id: Option<String>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct DeleteMessage {
        pub id: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct UpdateProfile {
        pub id: Option<String>,
        pub display_name: Option<String>,
        pub profile_picture: Option<String>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct WsChatMessage {
        pub chat_id: String,
        pub sender_id: String,
        pub message_id: String,
        pub message: String,
        pub reply_to: Option<String>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct WsCreateChat {
        pub chat_id: String,
        pub members: Vec<String>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct WsLeaveChat {
        pub chat_id: String,
        pub member: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct WsDeleteMessage {
        pub chat_id: String,
        pub message_id: String,
    }
}
