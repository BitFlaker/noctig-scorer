use iced::{Element, Length};
use iced::widget::{checkbox, column, space};

use crate::{Message, ProjectConfiguration};
use crate::formatting::theme;

pub fn view<'a>(project: &'a ProjectConfiguration) -> Element<'a, Message> {
    column![
        space().height(28.0),
        
        checkbox(project.filter_signal)
            .on_toggle(Message::ToggleFilterSignal)
            .size(19.0)
            .spacing(12.0)
            .style(theme::checkbox)
            .label("Filter signals"),
        checkbox(project.auto_align_signals)
            .on_toggle(Message::ToggleAutoAlignSignals)
            .size(19.0)
            .spacing(12.0)
            .style(theme::checkbox)
            .label("Auto align signals"),
        checkbox(project.clip_signal)
            .on_toggle(Message::ToggleClipSignal)
            .size(19.0)
            .spacing(12.0)
            .style(theme::checkbox)
            .label("Clip to signal range"),

        space().height(Length::Fill),
    ].spacing(20.0).into()
}