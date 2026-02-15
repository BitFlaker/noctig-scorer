use iced::widget::{Column, button, column, container, row};
use iced::{Element, Length, Pixels, Theme};
use iced_font_awesome::fa_icon_solid;
use iced::alignment::Vertical;
use iced::widget::text::Style;
use iced::widget::text;

use crate::formatting::theme;

#[derive(Debug, Clone)]
pub struct Collapsible<T> {
    title: String,
    subtitle: String,
    content_data: T,
    icons: [Option<String>; 2],
    is_expanded: bool
}

impl<T> Collapsible<T> {
    pub fn new(title: &str, subtitle: &str, content_data: T, icons: [Option<&str>; 2]) -> Self {
        Self {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            content_data,
            icons: icons.map(|i| i.map(|v| v.to_string())),
            is_expanded: false
        }
    }

    pub fn content_data(&self) -> &T {
        &self.content_data
    }

    pub fn subtitle(&self) -> &String {
        &self.subtitle
    }

    pub fn set_expanded(&mut self, is_expanded: bool) {
        self.is_expanded = is_expanded;
    }

    pub fn view<'a, Message>(
        &'a self,
        on_toggle_expand: impl Fn(bool) -> Message,
        view_body: impl Fn(&Self) -> Element<'a, Message>
    ) -> Element<'a, Message> where Message: Clone + 'a {
        if self.is_expanded {
            Column::with_children(vec![
                self.view_header(on_toggle_expand),
                container(
                    view_body(&self)
                )
                .style(theme::container_expanded_license)
                .padding([12.0, 16.0])
                .width(Length::Fill)
                .into()
            ]).into()
        } else {
            self.view_header(on_toggle_expand)
        }
    }

    fn view_header<'a, Message>(&self, on_toggle_expand: impl Fn(bool) -> Message) -> Element<'a, Message> where Message: Clone + 'a {
        button(
            row![
                fa_icon_solid(if self.is_expanded { "chevron-down" } else { "chevron-right" }).size(13.0).style(theme::text_secondary),
                column![
                    icon_prefix(&self.icons[0], &self.title, 16.0, theme::text_primary),
                    icon_prefix(&self.icons[1], &self.subtitle, 13.0, theme::text_secondary),
                ].spacing(6.0)
            ].align_y(Vertical::Center).spacing(16.0)
        )
        .padding([12.0, 20.0])
        .style(if self.is_expanded { theme::button_text_collapsible_expanded } else { theme::button_text_collapsible_collapsed })
        .on_press(on_toggle_expand(!self.is_expanded))
        .width(Length::Fill)
        .into()
    }
}

fn icon_prefix<'a, F, Message>(icon: &Option<String>, text_value: &String, size: impl Into<Pixels>, style: F) -> Element<'a, Message>
    where
Message: 'a,
F: Fn(&Theme) -> Style + 'a + Copy {
    let size = size.into();

    if let Some(icon) = icon {
        row![
            fa_icon_solid(icon).size(size).style(style),
            text(text_value.clone()).size(size).style(style),
        ].spacing(8.0).into()
    } else {
        text(text_value.clone()).size(size).style(style).into()
    }
}
