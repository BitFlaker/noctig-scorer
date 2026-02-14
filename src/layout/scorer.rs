use iced::widget::svg::Handle;
use iced::widget::{Canvas, Column, Id, Row, Space, center, column, container, row, scrollable, shader, space, stack, svg, text};
use iced::{Alignment, Element, Length, Padding};
use iced::alignment::Vertical;
use std::sync::LazyLock;
use itertools::Itertools;   // Required until `intersperse_with` is stabilized

use crate::formatting::font::REGULAR_BOLD;
use crate::formatting::theme::SPECTROGRAM_BORDER_WIDTH;
use crate::{CurrentProject, ICON_SECONDARY, key_legend};
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

    let spectrogram_view = view_spectrogram(&project);

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
            // when hovered -> Primary-Text-Color --> when clicked --> Open Menu
            container(row![
                svg(Handle::from_memory(ICON_SECONDARY.clone())).width(32.0),
                text("NoctiG Scorer").font(*REGULAR_BOLD).size(15.0).style(theme::text_secondary),
            ].align_y(Vertical::Center).spacing(8.0)),

            text(format!("{}*", project.project_name)).style(theme::text_primary),

            space().width(Length::Fill),

            // TODO: Add integrated windowing buttons
        ]
        .align_y(Vertical::Center)
        .spacing(16.0)
        .padding([12.0, 16.0])
        .width(Length::Fill),

        container(stack![
            spectrogram_view,

            container(space())
                .style(theme::container_spectrogram)
                .width(Length::Fill)
                .height(Length::Fill),
        ]).padding(Padding {
            left: 24.0,
            right: 24.0,
            top: 0.0,
            bottom: 8.0
        })
        .height(Length::Fixed(256.0)),

        Space::new().height(12.0),

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

fn view_spectrogram<'a>(project: &'a CurrentProject) -> Element<'a, Message> {
    if let Some(spectrogram) = &project.spectrogram {
        return container(shader(spectrogram)
            .width(Length::Fill)
            .height(Length::Fill)
        ).padding(SPECTROGRAM_BORDER_WIDTH - 0.5).into()
    }

    // No spectrogram available and not currently loading
    let Some(progress) = &project.loading_progress_spectrogram else {
        return space().into();
    };

    // Loading notice
    let loading = center(text("Loading ...").style(theme::text_secondary))
        .width(Length::Fill)
        .height(Length::Fill);

    // Empty progress bar
    if *progress as u16 == 0 {
        return loading.into();
    }

    // Loading progress with loading notice
    stack!(
        row![
            container(space())
                .style(theme::container_loading_spectrogram)
                .width(Length::FillPortion(*progress as u16))
                .height(Length::Fill),

            space()
                .width(Length::FillPortion(100 - *progress as u16))
                .height(Length::Fill),
        ],

        loading,
    ).into()
}
