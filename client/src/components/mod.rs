use iced::{
    widget::{button, container::Appearance, scrollable},
    Color, Font, Theme,
};

const ICON_FONT: Font = Font::with_name("Segoe MDL2 Assets");

fn truncate_message(message: String) -> String {
    if message.len() > 70 {
        message[..70].to_string() + "..."
    } else {
        message
    }
}

pub(crate) enum ChatButtonStyle {
    Open,
    Closed,
    SenderMessage,
    Delete,
}

impl button::StyleSheet for ChatButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Theme) -> button::Appearance {
        match self {
            ChatButtonStyle::Open => button::Appearance {
                border_radius: 8.0.into(),
                border_width: 2.0,
                border_color: style.palette().primary,
                background: Some(style.palette().background.into()),
                ..button::Appearance::default()
            },
            ChatButtonStyle::Closed => button::Appearance {
                border_radius: 8.0.into(),
                background: Some(style.palette().background.into()),
                ..button::Appearance::default()
            },
            ChatButtonStyle::SenderMessage => button::Appearance {
                border_radius: 8.0.into(),
                background: Some(style.palette().primary.into()),
                text_color: Color::WHITE.into(),
                ..button::Appearance::default()
            },
            ChatButtonStyle::Delete => button::Appearance {
                border_radius: 8.0.into(),
                background: Some(style.palette().danger.into()),
                text_color: Color::WHITE.into(),
                ..button::Appearance::default()
            },
        }
    }
}

struct ScrollableStyle;

impl scrollable::StyleSheet for ScrollableStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            border_color: Color::WHITE,
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            scroller: scrollable::Scroller {
                color: Color::from_rgba8(0, 0, 0, 0.2),
                border_radius: 8.0.into(),
                border_width: 0.0,
                border_color: Color::WHITE,
            },
        }
    }

    fn hovered(&self, style: &Self::Style, mouse_over_scrollbar: bool) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            border_color: Color::WHITE,
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            scroller: scrollable::Scroller {
                color: if mouse_over_scrollbar {
                    style.palette().primary
                } else {
                    Color::from_rgba8(0, 0, 0, 0.2)
                },
                border_radius: 8.0.into(),
                border_width: 0.0,
                border_color: Color::WHITE,
            },
        }
    }
}

fn style_outline(theme: &Theme) -> Appearance {
    Appearance {
        border_width: 2.0,
        border_color: theme.palette().primary,
        border_radius: 8.0.into(),
        background: Some(Color::from_rgb8(240, 240, 240).into()),
        ..Appearance::default()
    }
}

pub mod chat;
pub mod chat_list;
pub mod letter;
pub mod letter_list;