use iced::alignment::Vertical;
use iced::{Element, Length, Padding};
use iced::widget::{Column, button, column, container, row, scrollable, space, text};
use iced_font_awesome::{fa_icon, fa_icon_solid};

use crate::{Message, ProjectConfiguration, ProjectSignals};
use crate::formatting::{formatters, theme};

pub fn view<'a>(project: &'a ProjectConfiguration) -> Element<'a, Message> {
    column![
        space().height(30.0),

        row![
            text("Sources").size(14.0),

            space().width(Length::Fill),

            button(
                row![
                    fa_icon_solid("plus").size(15.0),
                    text("Add")
                ].align_y(Vertical::Center).spacing(12.0))
                .style(theme::button_secondary)
                .on_press(Message::LaunchBrowseImportSignal)
                .padding([8.0, 12.0]),
        ].align_y(Vertical::Bottom),

        space().height(8.0),

        container(
            scrollable(
                // TODO: Add some indicator that no signals are currently imported if data is empty
                Column::from_iter(project.data.iter().map(view_edf))
                    .spacing(8.0)
                    .padding(Padding {
                        left: 0.0,
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0
                    })
            )
        ).width(Length::Fill).height(Length::Fill),
    ].into()
}

fn view_edf<'a>(source: &ProjectSignals) -> Element<'a, Message>  {
    let start_time = formatters::date_time_string(source.timestamp);
    let duration = formatters::hms_separate(source.duration as u64);
    let signal_count = source.signal_count;

    container(
        container(row![
            space().width(4.0),

            fa_icon("window-maximize").size(16.0),  // TODO: EDF+ symbol

            column![
                text(source.name.clone()).style(theme::text_primary),
                text(source.path.clone()).style(theme::text_secondary).size(12.0),    // TODO: Make this ellipsis in case of too small of available space
                row![
                    text(format!("{} Signals", signal_count)).style(theme::text_secondary).size(12.0),
                    text(start_time).style(theme::text_secondary).size(12.0),
                    text(duration).style(theme::text_secondary).size(12.0),
                ].spacing(16.0)
            ].spacing(1.0).width(Length::Fill).padding([0.0, 24.0]),

            button(fa_icon_solid("xmark").color(theme::CLEAR_DARK_TEXT_SECONDARY).size(12.0))
                .height(36.0)
                .width(36.0)
                .padding(12.0)
                .on_press(Message::RemoveImportSignal(source.path.clone()))
                .style(theme::button_text),
        ].align_y(Vertical::Center).padding([4.0, 4.0]))
        ).padding([4.0, 16.0])
        .style(theme::container_secondary)
        .into()
}
