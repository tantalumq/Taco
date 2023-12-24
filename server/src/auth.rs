use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    routing::post,
    Json, Router,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::Serialize;
use structs::requests::LoginInfo;

use crate::{
    prisma::{self, read_filters::StringFilter, session, user},
    AppState,
};

#[derive(Clone, Serialize)]
pub(crate) struct Session {
    pub(crate) session_id: String,
    pub(crate) user_id: String,
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
    State(AppState { client, .. }): State<AppState>,
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

async fn log_in(
    State(AppState { client, .. }): State<AppState>,
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
        AppState { client, .. }: &AppState,
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

async fn log_out(State(AppState { client, .. }): State<AppState>, s: Session) {
    client
        .session()
        .delete(prisma::session::UniqueWhereParam::IdEquals(s.session_id))
        .exec()
        .await
        .unwrap();
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/log_in", post(log_in))
        .route("/log_out", post(log_out))
        .route("/register", post(register))
}
