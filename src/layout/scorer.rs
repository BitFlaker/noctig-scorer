use iced::widget::{Canvas, Column, Id, Row, Space, column, container, row, scrollable, space, stack, text};
use iced::{Alignment, Element, Length, Padding};
use iced::alignment::Vertical;
use std::sync::LazyLock;
use itertools::Itertools;   // Required until `intersperse_with` is stabilized

use crate::key_legend;
use crate::{Message, NoctiG, Stage};
use crate::formatting::{formatters, theme};
use crate::views::line_chart::Liner;

static LOG_SCROLLER_ID: LazyLock<Id> = LazyLock::new(Id::unique);
pub const SIGNAL_PADDING_VERTICAL: f32 = 12.0;

pub fn view(app: &NoctiG) -> Element<'_, Message> {
    let Some(project) = &app.current_project else {
        return space().into();
    };
    let Some(scorings) = &project.scorings else {
        return space().into();
    };

    let mut index = 0;
    let signals = Column::from_vec(
        project.readers.iter().map(|reader| {
            let base_index = index;
            index += reader.signal_count();

            Column::from_iter(
                reader.get_chart_signals().into_iter().map(|signal|
                    Liner::from_chart_signal(signal, base_index, app.draw_ranges, project.project.epochs_before_current, project.project.epochs_after_current)).map(|l|
                        Canvas::new(l)
                            .width(Length::Fill)
                            .height(Length::Fixed(100.0 + 2.0 * SIGNAL_PADDING_VERTICAL))
                            .into()
                )
            ).into()
        }
    ).collect::<Vec<_>>()).width(Length::Fill);

    let default_reader = project.readers.iter().max_by(|r1, r2| r1.get_epoch_count().cmp(&r2.get_epoch_count())).unwrap();

    let current_seg_n = default_reader.get_window_start_epoch();
    let start_segment = current_seg_n.saturating_sub(project.project.epochs_before_current as u64);
    let end_segment = current_seg_n + project.project.epochs_after_current as u64 + 1;

    // Get the amount of visible offset placeholder epochs
    let underflow = (project.project.epochs_before_current as u64).saturating_sub(current_seg_n);
    let max_epoch = default_reader.get_epoch_count();

    // TODO: The start timestamp has to be taken from the least offset reader (if there are multiple,
    // take the earliest / allow user to choose (some recording devices might have wrong timestamp due
    // to clock drift, etc) there might be a trusted source file too though and then we would have to calculate
    // the actual start timestamp by reducing the timestamp by the offset between the earlist record and the selected one)
    // Potentially prompt the user after first opening the project (only when there are multiple different timestamps) which one to use
    let time_frame = default_reader.current_timeframe();
    let start_time = default_reader.start_timestamp();
    let current_timespan = formatters::TIME_FORMATTERS.get(app.window_time_formatter_index).unwrap()(start_time, time_frame.0, time_frame.1);

    let stages = Row::from_iter((start_segment..(end_segment + underflow)).map(move |i| {
        if i < underflow || i >= max_epoch {
            return space().width(Length::FillPortion((end_segment - start_segment) as u16)).into();
        }

        let stage = scorings.values.get(&(i - underflow)).cloned().unwrap_or(Stage::Unset);

        container(
            container(
                text(stage.map_str()).style(Stage::foreground(stage.clone())).size(20.0)
            ).style(Stage::background(stage)).padding([4.0, 0.0]).width(Length::Fill).align_x(Alignment::Center)
        ).padding([0.0, 16.0]).width(Length::FillPortion((end_segment - start_segment) as u16)).into()
    }));

    column![
        row![
            text("NoctiG Scorer"),
            text("SomeProject.ngp*"),
            text("Sleep-Scoring"),

            space().width(Length::Fill),

            // TODO: Add integrated windowing buttons and improve design
        ].spacing(8.0).width(Length::Fill),

        Space::new().height(24),

        stack!(
            scrollable(
                column![
                    signals,

                    // Bottom padding to make place for floating stage indicators
                    space().height(24.0 + 32.0)
                ]
            ).id(LOG_SCROLLER_ID.clone()).anchor_top().width(Length::Fill).height(Length::Fill),

            column![
                Space::new().height(Length::Fill),
                stages.width(Length::Fill),
                Space::new().height(12),
            ].width(Length::Fill),
        ).width(Length::Fill),

        // TODO: Add full markers and annotation timeline

        container(
            column![
                // Status stroke
                container(Space::new().height(1.0)).width(Length::Fill).style(theme::stroke),

                row![

                    // Legend of most relevant shortcuts
                    row![
                        key_legend!(["W", "R", "1", "2", "3", "Del"], "Set rating"),
                        key_legend!(["⏴", "⏵"], "Move axis"),
                        key_legend!("T", "Time format"),
                        key_legend!("H", "Help"),
                    ].spacing(16.0).align_y(Vertical::Center),

                    // Space divider
                    Space::new().width(Length::Fill),

                    // Current time fragment
                    text(current_timespan).style(theme::text_secondary).size(18.0)
                ].padding(Padding { left: 8.0, top: 6.0, right: 8.0, bottom: 6.0 })
            ]
        ).style(theme::status_bar)
    ].width(Length::Fill).into()
}
