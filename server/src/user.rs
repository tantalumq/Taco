use std::collections::HashSet;

use crate::{
    option_vec,
    prisma::{chat, message, read_filters::StringFilter, user},
    AppState, WsMessage,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use structs::requests::{
    ChatWithMembers, CreateChat, CreateMessage, DeleteMessage, LeaveChat, UpdateProfile,
    UserStatus, WsChatMessage, WsCreateChat, WsDeleteMessage, WsLeaveChat,
};

use crate::Session;

user::select!(user_status {
    id
    display_name
    profile_picture
    online
});

async fn get_user_status(
    State(AppState { client, .. }): State<AppState>,
    Path(user_id): Path<String>,
) -> Json<Option<UserStatus>> {
    Json(
        client
            .user()
            .find_unique(user::UniqueWhereParam::IdEquals(user_id))
            .select(user_status::select())
            .exec()
            .await
            .unwrap()
            .map(|status| UserStatus {
                id: status.id,
                display_name: status.display_name,
                profile_picture: status.profile_picture,
                online: status.online,
            }),
    )
}

user::select!(chat_with_members {
    chats: select {
        id
        members: select {
            id
        }
    }
});

async fn get_user_chats(
    State(AppState { client, .. }): State<AppState>,
    session: Session,
) -> Json<Vec<ChatWithMembers>> {
    Json(
        client
            .user()
            .find_unique(user::UniqueWhereParam::IdEquals(session.user_id))
            .select(chat_with_members::select())
            .exec()
            .await
            .unwrap()
            .unwrap()
            .chats
            .into_iter()
            .map(|chat| ChatWithMembers {
                id: chat.id,
                members: chat.members.into_iter().map(|user| user.id).collect(),
            })
            .collect(),
    )
}

async fn create_chat(
    State(AppState {
        client,
        message_sender,
    }): State<AppState>,
    session: Session,
    Json(chat): Json<CreateChat>,
) -> String {
    let chat = client
        .chat()
        .create(vec![chat::SetParam::ConnectMembers(vec![
            user::UniqueWhereParam::IdEquals(session.user_id),
            user::UniqueWhereParam::IdEquals(chat.other_members),
        ])])
        .select(chat::select!({
            id
            members: select {
                id
            }
        }))
        .exec()
        .await
        .unwrap();

    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(chat.members.iter().map(|member| member.id.clone())),
            data: serde_json::to_value(WsCreateChat {
                chat_id: chat.id.clone(),
                members: chat.members.into_iter().map(|member| member.id).collect(),
            })
            .unwrap(),
        })
        .unwrap();

    chat.id
}

async fn leave_chat(
    State(AppState {
        client,
        message_sender,
    }): State<AppState>,
    session: Session,
    Json(chat): Json<LeaveChat>,
) -> Result<(), (StatusCode, &'static str)> {
    let chat = client
        .chat()
        .find_unique(chat::UniqueWhereParam::IdEquals(chat.chat_id))
        .select(chat::select!({
            id
            members: select {
                id
            }
        }))
        .exec()
        .await
        .unwrap()
        .ok_or((StatusCode::NOT_FOUND, "Chat was not found"))?;
    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(chat.members.into_iter().map(|member| member.id)),
            data: serde_json::to_value(WsLeaveChat {
                chat_id: chat.id,
                member: session.user_id,
            })
            .unwrap(),
        })
        .unwrap();
    Ok(())
}

async fn get_messages(
    State(AppState { client, .. }): State<AppState>,
    session: Session,
    Path(chat_id): Path<String>,
) -> Result<Json<Vec<message::Data>>, (StatusCode, &'static str)> {
    let chat = client
        .chat()
        .find_first(vec![
            chat::WhereParam::Id(StringFilter::Equals(chat_id)),
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
    State(AppState {
        client,
        message_sender,
    }): State<AppState>,
    session: Session,
    Json(message): Json<CreateMessage>,
) -> String {
    let message = client
        .message()
        .create(
            chat::UniqueWhereParam::IdEquals(message.chat_id),
            message.content,
            user::UniqueWhereParam::IdEquals(session.user_id.clone()),
            option_vec![message
                .reply_to_id
                .map(
                    |id| message::SetParam::ConnectReplyTo(message::UniqueWhereParam::IdEquals(id))
                )],
        )
        .include(message::include!({
            chat: select {
                members: select {
                    id
                }
            }
        }))
        .exec()
        .await
        .unwrap();

    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(
                message.chat.members.into_iter().map(|member| member.id),
            ),
            data: serde_json::to_value(WsChatMessage {
                chat_id: message.chat_id,
                sender_id: session.user_id,
                message: message.content,
                message_id: message.id.clone(),
                reply_to: message.reply_id,
            })
            .unwrap(),
        })
        .unwrap();

    message.id
}

async fn delete_message(
    State(AppState {
        client,
        message_sender,
    }): State<AppState>,
    Json(message): Json<DeleteMessage>,
) {
    let message = client
        .message()
        .delete(message::UniqueWhereParam::IdEquals(message.id))
        .include(message::include!({
            chat: select {
                id
                members: select {
                    id
                }
            }
        }))
        .exec()
        .await
        .unwrap();
    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(
                message.chat.members.into_iter().map(|member| member.id),
            ),
            data: serde_json::to_value(WsDeleteMessage {
                chat_id: message.chat.id,
                message_id: message.id,
            })
            .unwrap(),
        })
        .unwrap();
}

async fn update_profile(
    State(AppState { client, .. }): State<AppState>,
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
        .route("/chats", get(get_user_chats))
        .route("/status/:user_id", get(get_user_status))
        .route("/messages/:chat_id", get(get_messages))
        .route("/update_profile", post(update_profile))
        .route("/create_message", post(create_message))
        .route("/create_chat", post(create_chat))
        .route("/leave_chat", post(leave_chat))
        .route("/delete_message", post(delete_message))
}
