use chrono::{DateTime, Local};
use iced::{Element, Length, Padding};
use iced::widget::{Column, button, center, column, container, row, scrollable, space, stack, text, text_input};
use iced::alignment::Vertical;
use iced_font_awesome::{fa_icon, fa_icon_solid};

use crate::database::types::RecentProject;
use crate::{Message, NoctiG};
use crate::formatting::theme;

pub fn view(app: &NoctiG) -> Element<'_, Message> {
    column![
        row![
            stack![
                text_input("Search", &app.search_text)  // TODO: Get magnifying glass icon in there
                    .style(theme::text_input_secondary)
                    .on_input(Message::ProjectSearchChanged)
                    .width(Length::Fill)
                    .padding(Padding {
                        left: 38.0,
                        top: 8.0,
                        right: 12.0,
                        bottom: 8.0
                    }),

                container(
                    fa_icon_solid("magnifying-glass")
                        .style(theme::text_secondary)
                        .size(14.0)
                ).padding(Padding {
                    left: 12.0,
                    top: 0.0,
                    right: 0.0,
                    bottom: 1.0
                })
                .height(Length::Fill)
                .align_y(Vertical::Center)
            ],

            button(row![
                fa_icon_solid("plus").size(15.0),
                text("New project")
            ].align_y(Vertical::Center).spacing(12.0))
                .style(theme::button_primary)
                .on_press(Message::CreateProjectWizard)
                .padding([8.0, 12.0]),

            button(row![
                fa_icon_solid("folder-open").size(15.0),
                text("Open")
            ].align_y(Vertical::Center).spacing(12.0))
                .style(theme::button_secondary)
                .on_press(Message::LaunchOpenProject)
                .padding([8.0, 12.0])
        ].spacing(8.0),

        space().height(12.0),

        // Recent projects list
        container(
            view_recent_projects(app)
        ).style(theme::container_recent_projects)
            .width(Length::Fill)
            .height(Length::Fill)

    ].into()
}

fn view_recent_projects<'a>(app: &NoctiG) -> Element<'_, Message> {
    if let Some(filtered) = &app.filtered_recent_projects {
        if filtered.is_empty() {
            center(text("No projects found for search").style(theme::text_secondary)).into()
        }
        else {
            get_recents_scroller(filtered)
        }
    } else if !app.recent_projects.is_empty() {
        get_recents_scroller(&app.recent_projects)
    } else {
        center(text("No recent projects").style(theme::text_secondary)).into()
    }
}

fn get_recents_scroller<'a>(projects: &Vec<RecentProject>) -> Element<'a, Message> {
    scrollable(
        column![
            space().height(2.0),
            Column::from_iter(projects.iter().map(view_recent_project)),
            space().height(2.0),
        ].padding([8.0, 12.0]),
    ).into()
}

fn view_recent_project<'a>(project: &RecentProject) -> Element<'a, Message>  {
    let local_time: DateTime<Local> = DateTime::from(project.last_opened);

    button(
        container(row![
            space().width(4.0),

            fa_icon("window-maximize").size(16.0),

            column![
                text(project.name.to_string()).style(theme::text_primary),
                text(project.path.to_string()).style(theme::text_secondary).size(12.0)    // TODO: Make this ellipsis in case of too small of available space
            ].spacing(1.0).width(Length::Fill).padding([0.0, 24.0]),

            text(local_time.format("%x %X").to_string()).style(theme::text_tertiary).size(12.0),

            space().width(4.0),
        ].align_y(Vertical::Center).padding([12.0, 4.0]))
    ).on_press(Message::OpenProjectPath(project.path.to_string()))
        .style(theme::button_text_secondary)
        .into()
}
