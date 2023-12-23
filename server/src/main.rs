use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequestParts, Query, State},
    http::{request::Parts, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use prisma::{chat, message, read_filters::StringFilter, session, user};
use serde::{Deserialize, Serialize};

#[allow(warnings, unused)]
mod prisma;

#[derive(Clone)]
struct AppState {
    client: Arc<prisma::PrismaClient>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/status", get(get_user_status))
        .route("/chats", get(get_user_chats))
        .route("/create_chat", post(create_chat))
        .route("/get_messages", get(get_messages))
        .with_state(AppState {
            client: Arc::new(prisma::new_client().await.unwrap()),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct UserQuery {
    id: String,
}

user::select!(user_status {
    id
    display_name
    profile_picture
    online
});

async fn get_user_status(
    State(AppState { client }): State<AppState>,
    Query(UserQuery { id }): Query<UserQuery>,
) -> Json<Option<user_status::Data>> {
    Json(
        client
            .user()
            .find_unique(user::UniqueWhereParam::IdEquals(id))
            .select(user_status::select())
            .exec()
            .await
            .unwrap(),
    )
}

#[derive(Clone, Serialize)]
struct Session {
    session_id: String,
    user_id: String,
}

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

fn hash<T: AsRef<str>>(s: T) -> String {
    sha256::digest(s.as_ref())
}

fn get_now_msk() -> DateTime<FixedOffset> {
    const HOUR: i32 = 3600;
    Utc::now().with_timezone(&FixedOffset::east_opt(3 * HOUR).unwrap())
}

fn get_session_expiry() -> DateTime<FixedOffset> {
    const SESSION_DURATION_DAYS: i64 = 10;
    get_now_msk() + Duration::days(SESSION_DURATION_DAYS)
}

/// Returns session id
async fn create_session(client: Arc<prisma::PrismaClient>, user_id: String) -> String {
    let session = client
        .session()
        .create(
            get_session_expiry(),
            user::UniqueWhereParam::IdEquals(user_id),
            vec![],
        )
        .exec()
        .await
        .unwrap();

    session.id
}

async fn register(
    State(AppState { client }): State<AppState>,
    Json(info): Json<LoginInfo>,
) -> Json<Session> {
    client
        .user()
        .create(
            info.username.clone(),
            hash(info.password),
            info.username.clone(),
            vec![],
        )
        .exec()
        .await
        .unwrap();

    let session_id = create_session(client, info.username.clone()).await;
    Json(Session {
        user_id: info.username,
        session_id,
    })
}

async fn login(
    State(AppState { client }): State<AppState>,
    Json(info): Json<LoginInfo>,
) -> Json<Option<Session>> {
    let user = client
        .user()
        .find_first(vec![
            user::WhereParam::Id(StringFilter::Equals(info.username)),
            user::WhereParam::Password(StringFilter::Equals(hash(info.password))),
        ])
        .exec()
        .await
        .unwrap();

    if let Some(user) = user {
        let session_id = create_session(client, user.id.clone()).await;
        Json(Some(Session {
            user_id: user.id,
            session_id,
        }))
    } else {
        Json(None)
    }
}

/// Returns None if the session was not found or is expired, otherwise renews the session and returns the user id
async fn check_session(client: Arc<prisma::PrismaClient>, session_id: String) -> Option<String> {
    let session = client
        .session()
        .find_unique(session::UniqueWhereParam::IdEquals(session_id.clone()))
        .exec()
        .await
        .unwrap();

    if let Some(session) = session {
        if session.expires_at < get_now_msk() {
            client
                .session()
                .delete(session::UniqueWhereParam::IdEquals(session_id))
                .exec()
                .await
                .unwrap();

            None
        } else {
            client
                .session()
                .update(
                    session::UniqueWhereParam::IdEquals(session_id),
                    vec![session::SetParam::SetExpiresAt(get_session_expiry())],
                )
                .exec()
                .await
                .unwrap();

            Some(session.user_id)
        }
    } else {
        None
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        req: &mut Parts,
        AppState { client }: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorization = req
            .headers
            .get("Authorization")
            .ok_or((StatusCode::BAD_REQUEST, "Missing `Authorization` header"))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid characters in `Authorization` header",
                )
            })?;

        match authorization.split_once(' ') {
            Some((name, session_id)) if name == "Bearer" => {
                check_session(client.clone(), session_id.into())
                    .await
                    .ok_or((StatusCode::BAD_REQUEST, "Invalid session"))
                    .map(|user_id| Session {
                        session_id: session_id.into(),
                        user_id,
                    })
            }
            _ => Err((
                StatusCode::BAD_REQUEST,
                "Invalid `Authorization` header value, Bearer must be used",
            )),
        }
    }
}

user::select!(chat_with_members {
    chats: select {
        id
        members
    }
});

async fn get_user_chats(
    State(AppState { client }): State<AppState>,
    session: Session,
) -> Json<Vec<chat_with_members::chats::Data>> {
    Json(
        client
            .user()
            .find_unique(user::UniqueWhereParam::IdEquals(session.user_id))
            .select(chat_with_members::select())
            .exec()
            .await
            .unwrap()
            .unwrap()
            .chats,
    )
}

#[derive(Deserialize)]
struct CreateChat {
    other_member: String,
}

async fn create_chat(
    State(AppState { client }): State<AppState>,
    session: Session,
    Json(chat): Json<CreateChat>,
) -> String {
    client
        .chat()
        .create(vec![chat::SetParam::ConnectMembers(vec![
            user::UniqueWhereParam::IdEquals(session.user_id),
            user::UniqueWhereParam::IdEquals(chat.other_member),
        ])])
        .exec()
        .await
        .unwrap()
        .id
}

#[derive(Deserialize)]
struct GetChatMessages {
    chat_id: String,
}

async fn get_messages(
    State(AppState { client }): State<AppState>,
    session: Session,
    Json(chat): Json<GetChatMessages>,
) -> Result<Json<Vec<message::Data>>, (StatusCode, &'static str)> {
    let chat = client
        .chat()
        .find_first(vec![
            chat::WhereParam::Id(StringFilter::Equals(chat.chat_id)),
            chat::WhereParam::MembersSome(vec![user::WhereParam::Id(StringFilter::Equals(
                session.user_id,
            ))]),
        ])
        .select(chat::select!({ messages }))
        .exec()
        .await
        .unwrap()
        .ok_or((StatusCode::NOT_FOUND, "Chat not found"))?;

    Ok(Json(chat.messages))
}
