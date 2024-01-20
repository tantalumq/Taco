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

use chrono::Utc;
use prisma_client_rust::Direction;
use structs::requests::{
    ChatWithMembers, CreateChat, CreateMessage, DeleteMessage, LeaveChat, UpdateProfile,
    UserStatus, WsChatMessage, WsCreateChat, WsDeleteMessage, WsLeaveChat, WsMessageData,
};

use crate::Session;

async fn get_user_status(
    State(AppState { client, .. }): State<AppState>,
    Path(user_id): Path<String>,
) -> Json<Option<UserStatus>> {
    Json(
        client
            .user()
            .find_unique(user::UniqueWhereParam::IdEquals(user_id))
            .select(user::select!({
                id
                display_name
                profile_picture
                online
            }))
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

async fn get_user_chats(
    State(AppState { client, .. }): State<AppState>,
    session: Session,
) -> Json<Vec<ChatWithMembers>> {
    Json(
        client
            .user()
            .find_unique(user::UniqueWhereParam::IdEquals(session.user_id))
            .select(user::select!({
                chats: select {
                    id
                    members: select {
                        id
                    }
                    last_updated
                }
            }))
            .exec()
            .await
            .unwrap()
            .unwrap()
            .chats
            .into_iter()
            .map(|chat| ChatWithMembers {
                id: chat.id,
                members: chat.members.into_iter().map(|user| user.id).collect(),
                last_updated: chat.last_updated.into(),
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
    Json(create_chat): Json<CreateChat>,
) -> Result<Json<ChatWithMembers>, (StatusCode, &'static str)> {
    if create_chat.other_members == session.user_id {
        return Err((StatusCode::CONFLICT, "Нельзя создать чат с самим собой!"));
    }

    let user = client
        .user()
        .find_unique(user::UniqueWhereParam::IdEquals(
            create_chat.other_members.clone(),
        ))
        .exec()
        .await
        .unwrap();
    if user.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Такого пользователя не существует!",
        ));
    }

    let chat = client
        .chat()
        .create(vec![chat::SetParam::ConnectMembers(vec![
            user::UniqueWhereParam::IdEquals(session.user_id),
            user::UniqueWhereParam::IdEquals(create_chat.other_members),
        ])])
        .select(chat::select!({
            id
            members: select {
                id
            }
            last_updated
        }))
        .exec()
        .await
        .unwrap();

    let member_ids: Vec<String> = chat.members.into_iter().map(|member| member.id).collect();

    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(member_ids.clone()),
            data: WsMessageData::CreateChat(WsCreateChat {
                chat_id: chat.id.clone(),
                members: member_ids.clone(),
            }),
        })
        .unwrap();

    Ok(Json(ChatWithMembers {
        id: chat.id,
        members: member_ids,
        last_updated: chat.last_updated.into(),
    }))
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
        .ok_or((StatusCode::NOT_FOUND, "Чат не найден!"))?;
    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(chat.members.into_iter().map(|member| member.id)),
            data: WsMessageData::LeaveChat(WsLeaveChat {
                chat_id: chat.id,
                member: session.user_id,
            }),
        })
        .unwrap();
    Ok(())
}

async fn get_messages(
    State(AppState { client, .. }): State<AppState>,
    session: Session,
    Path(chat_id): Path<String>,
) -> Result<Json<Vec<WsChatMessage>>, (StatusCode, &'static str)> {
    let chat = client
        .chat()
        .find_first(vec![
            chat::WhereParam::Id(StringFilter::Equals(chat_id)),
            chat::WhereParam::MembersSome(vec![user::WhereParam::Id(StringFilter::Equals(
                session.user_id,
            ))]),
        ])
        .select(chat::select!({
            messages(vec![]).order_by(message::created_at::order(Direction::Asc))
        }))
        .exec()
        .await
        .unwrap()
        .ok_or((StatusCode::NOT_FOUND, "Чат не найден"))?;

    Ok(Json(
        chat.messages
            .into_iter()
            .map(|message| WsChatMessage {
                chat_id: message.chat_id,
                sender_id: message.user_id,
                message: message.content,
                message_id: message.id,
                reply_to: message.reply_id,
                created_at: message.created_at.into(),
            })
            .collect(),
    ))
}

async fn create_message(
    State(AppState {
        client,
        message_sender,
    }): State<AppState>,
    session: Session,
    Json(message): Json<CreateMessage>,
) -> Json<String> {
    let (message, _) = client
        ._batch((
            client
                .message()
                .create(
                    chat::UniqueWhereParam::IdEquals(message.chat_id.clone()),
                    message.content,
                    user::UniqueWhereParam::IdEquals(session.user_id.clone()),
                    option_vec![message
                        .reply_to_id
                        .map(|id| message::SetParam::ConnectReplyTo(
                            message::UniqueWhereParam::IdEquals(id)
                        ))],
                )
                .include(message::include!({
                    chat: select {
                        members: select {
                            id
                        }
                    }
                })),
            client.chat().update(
                chat::UniqueWhereParam::IdEquals(message.chat_id),
                vec![chat::SetParam::SetLastUpdated(Utc::now().into())],
            ),
        ))
        .await
        .unwrap();

    message_sender
        .send(WsMessage {
            recipient_ids: HashSet::from_iter(
                message.chat.members.into_iter().map(|member| member.id),
            ),
            data: WsMessageData::ChatMessage(WsChatMessage {
                chat_id: message.chat_id,
                sender_id: session.user_id,
                message: message.content,
                message_id: message.id.clone(),
                reply_to: message.reply_id,
                created_at: message.created_at.into(),
            }),
        })
        .unwrap();

    Json(message.id)
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
            data: WsMessageData::DeleteMessage(WsDeleteMessage {
                chat_id: message.chat.id,
                message_id: message.id,
            }),
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
