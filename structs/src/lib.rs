pub mod requests {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct UserQuery {
        pub id: String,
    }

    #[derive(Clone, Serialize)]
    pub struct Session {
        pub session_id: String,
        pub user_id: String,
    }

    #[derive(Deserialize)]
    pub struct LoginInfo {
        pub username: String,
        pub password: String,
    }

    #[derive(Deserialize)]
    pub struct CreateChat {
        pub other_member: String,
    }
    #[derive(Deserialize)]
    pub struct GetChatMessages {
        pub chat_id: String,
    }
    #[derive(Deserialize)]
    pub struct CreateMessage {
        pub chat_id: String,
        pub message_id: String,
        pub content: String,
        pub reply_to_id: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct UpdateProfile {
        pub id: Option<String>,
        pub display_name: Option<String>,
        pub profile_picture: Option<String>,
    }
}
