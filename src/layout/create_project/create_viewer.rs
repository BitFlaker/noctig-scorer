use iced::alignment::Vertical;
use iced::{Element, Length, Padding};
use iced::widget::{button, column, container, row, space, svg, text};
use iced::widget::svg::Handle;

use crate::{CreatePage, ICON, Message, NoctiG, Page, WindowType};
use crate::layout::create_project::{project_data, project_details, project_processing};
use crate::formatting::{font, theme};

pub fn view<'a>(app: &'a NoctiG, page: &'a CreatePage) -> Element<'a, Message> {
    let Some(ref project) = app.project_creation else {
        return space().into();
    };

    container(row![
        // Left navigation sidebar
        container(column![
            view_branding(),

            space().height(32),

            button(row![
                    container(text("1").size(14.0).style(|theme| theme::text_counter(theme, *page == CreatePage::Project))).center(24.0).style(|theme| theme::container_counter(theme, *page == CreatePage::Project)),
                    text("Project details")
                ].spacing(12.0).align_y(Vertical::Center))
                .style(|theme, status| theme::button_current_create_page(theme, status, *page == CreatePage::Project))
                .on_press(Message::SwitchPage(Page::CreateProject(CreatePage::Project)))
                .padding(12.0)
                .width(Length::Fill),
            button(row![
                    container(text("2").size(14.0).style(|theme| theme::text_counter(theme, *page == CreatePage::Data))).center(24.0).style(|theme| theme::container_counter(theme, *page == CreatePage::Data)),
                    text("Signals")
                ].spacing(12.0).align_y(Vertical::Center))
                .style(|theme, status| theme::button_current_create_page(theme, status, *page == CreatePage::Data))
                .on_press(Message::SwitchPage(Page::CreateProject(CreatePage::Data)))
                .padding(12.0)
                .width(Length::Fill),
            button(row![
                    container(text("3").size(14.0).style(|theme| theme::text_counter(theme, *page == CreatePage::Processing))).center(24.0).style(|theme| theme::container_counter(theme, *page == CreatePage::Processing)),
                    text("Processing")
                ].spacing(12.0).align_y(Vertical::Center))
                .style(|theme, status| theme::button_current_create_page(theme, status, *page == CreatePage::Processing))
                .on_press(Message::SwitchPage(Page::CreateProject(CreatePage::Processing)))
                .padding(12.0)
                .width(Length::Fill),

            space().height(Length::Fill),

            container(
                row![
                    button(text("Help").size(12.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::OpenWindow(WindowType::Licenses))
                        .padding([4.0, 8.0]),

                    container(text("•")
                        .style(theme::text_secondary)
                        .size(14.0)
                    ).padding(Padding { left: 0.0, top: 3.0, right: 0.0, bottom: 0.0 }),

                    button(text("Edit Presets").size(12.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::OpenWindow(WindowType::Licenses))
                        .padding([4.0, 8.0])
                ]
            ).padding([6.0, 8.0]),
        ].spacing(6.0)).width(Length::Fixed(256.0)).padding(12.0),

        // Main center content
        container(row![
            column![
                space().height(12.0),
                text(get_title(page)).size(32.0),
                space().height(4.0),

                text("Create a new project and import signals from EDF files. Then adjust signal processing and other miscellaneous options for the project.").style(theme::text_secondary).size(14.0),

                match page {
                    CreatePage::Project => project_details::view(project),
                    CreatePage::Data => project_data::view(project),
                    CreatePage::Processing => project_processing::view(project),
                },

                space().height(16.0),

                // Informational footer links
                row![
                    space().width(Length::Fill),

                    button(text(get_action_secondary(page)))
                        .style(theme::button_text)
                        .on_press(get_next_page(page, true))
                        .padding([8.0, 12.0]),
                    button(text(get_action_primary(page)))
                        .style(theme::button_primary)
                        .on_press(get_next_page(page, false))
                        .padding([8.0, 12.0]),
                ].spacing(8.0),

                space().height(12.0)
            ],
            space().width(24.0)
        ]).width(Length::Fill).height(Length::Fill).padding(12.0)
    ]).into()
}

fn view_branding<'a>() -> Element<'a, Message>  {
    container(
        row![
            svg(Handle::from_memory(ICON)).width(56.0),

            column![
                text("NoctiG Scorer").font(*font::REGULAR_BOLD),
                text("v0.1.0").style(theme::text_secondary).size(12.0)
            ]
        ].spacing(12.0).align_y(Vertical::Center).padding([0.0, 8.0])
    ).height(64.0).align_y(Vertical::Center).into()
}

fn get_title(page: &CreatePage) -> String {
    return match page {
        CreatePage::Project => "Project Details",
        CreatePage::Data => "Import Data",
        CreatePage::Processing => "Processing Options"
    }.to_string()
}

fn get_action_secondary(page: &CreatePage) -> String {
    return match page {
        CreatePage::Project => "Cancel",
        _ => " Back "
    }.to_string()
}

fn get_action_primary(page: &CreatePage) -> String {
    return match page {
        CreatePage::Processing => "Create",
        _ => " Next "
    }.to_string()
}

fn get_next_page(page: &CreatePage, reverse: bool) -> Message {
    return match page {
        CreatePage::Project if !reverse => Message::SwitchPage(Page::CreateProject(CreatePage::Data)),
        CreatePage::Project => Message::CancelCreateProject,
        CreatePage::Data if !reverse => Message::SwitchPage(Page::CreateProject(CreatePage::Processing)),
        CreatePage::Data => Message::SwitchPage(Page::CreateProject(CreatePage::Project)),
        CreatePage::Processing if !reverse => Message::CreateProject,
        CreatePage::Processing => Message::SwitchPage(Page::CreateProject(CreatePage::Data))
    }
}
