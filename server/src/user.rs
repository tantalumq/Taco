use std::sync::Arc;

use crate::{
    option_vec,
    prisma::{self, chat, message, read_filters::StringFilter, user},
    AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use structs::requests::{CreateChat, CreateMessage, GetChatMessages, UpdateProfile, UserQuery};

use crate::Session;

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

async fn create_chat(
    client: Arc<prisma::PrismaClient>,
    session: Session,
    chat: CreateChat,
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

async fn create_message(
    client: Arc<prisma::PrismaClient>,
    session: Session,
    message: CreateMessage,
) -> String {
    client
        .message()
        .create(
            chat::UniqueWhereParam::IdEquals(message.chat_id),
            message.content,
            user::UniqueWhereParam::IdEquals(session.user_id),
            option_vec![message
                .reply_to_id
                .map(
                    |id| message::SetParam::ConnectReplyTo(message::UniqueWhereParam::IdEquals(id))
                )],
        )
        .exec()
        .await
        .unwrap()
        .id
}

async fn update_profile(
    State(AppState { client }): State<AppState>,
    session: Session,
    Json(update_profile): Json<UpdateProfile>,
) {
    client
        .user()
        .update(
            user::UniqueWhereParam::IdEquals(session.user_id),
            option_vec![
                update_profile.id.map(user::SetParam::SetId),
                update_profile
                    .display_name
                    .map(user::SetParam::SetDisplayName),
                Some(user::SetParam::SetProfilePicture(
                    update_profile.profile_picture
                )),
            ],
        )
        .exec()
        .await
        .unwrap();
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_user_status))
        .route("/chats", get(get_user_chats))
        .route("/messages", get(get_messages))
        .route("/update_profile", post(update_profile))
}
