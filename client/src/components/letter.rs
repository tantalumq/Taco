use iced::{
    theme::Button,
    widget::{button, column, container, row, text},
    Color, Element, Length,
};
use structs::{requests::WsChatMessage, DateTime, Local};

use crate::components::truncate_message;

use super::{letter_list::LetterList, ChatButtonStyle};

#[derive(Clone)]
pub struct Letter(pub WsChatMessage);

#[derive(Debug, Clone, PartialEq)]
pub enum LetterMessage {
    ReplyStarted,
    LetterDelete,
}

impl Letter {
    pub fn update(&mut self, message: LetterMessage) {
        match message {
            _ => {}
        }
    }

    pub fn view(
        &self,
        letter_list: &LetterList,
        current_user_id: String,
    ) -> Element<LetterMessage> {
        // TODO: put your messages on the right
        let nickname = self.0.sender_id.clone();
        let reply_message = self
            .0
            .reply_to
            .clone()
            .and_then(|id| letter_list.messages.get(&id));

        let message_column = if let Some(reply_message) = reply_message {
            column![text(&format!(
                "↱ {}: {}",
                reply_message.0.sender_id,
                truncate_message(reply_message.0.message.clone(), 80)
            ))
            .size(12),]
        } else {
            column![]
        };

        let local_created_at: DateTime<Local> = self.0.created_at.into();

        let message_row = row![button(
            column![
                text(nickname.clone()),
                text(self.0.message.clone()),
                text(local_created_at.format("%d/%m/%Y %H:%M").to_string())
                    .style(if nickname == current_user_id {
                        Color::from_rgba8(255, 255, 255, 0.5)
                    } else {
                        Color::from_rgba8(0, 0, 0, 0.5)
                    })
                    .size(11)
            ]
            .spacing(5),
        )
        .padding(10)
        .style(Button::Custom(Box::new(if nickname == current_user_id {
            ChatButtonStyle::SenderMessage
        } else {
            ChatButtonStyle::Closed
        })))
        .on_press(LetterMessage::ReplyStarted),]
        .align_items(iced::Alignment::Center);
        container(
            message_column
                .push(
                    if nickname == current_user_id {
                        message_row.push(
                            container(
                                button("×")
                                    .padding([0, 10])
                                    .style(Button::Custom(Box::new(ChatButtonStyle::Delete)))
                                    .on_press(LetterMessage::LetterDelete),
                            )
                            .center_y(),
                        )
                    } else {
                        message_row
                    }
                    .spacing(15),
                )
                .spacing(8),
        )
        .padding([0, 10, 0, 0])
        .width(Length::Fill)
        .into()
    }
}
