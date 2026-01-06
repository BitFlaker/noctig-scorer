use iced::{Element, Length, Padding};
use iced::widget::{button, stack, column, container, row, scrollable, space, svg, text, text_input};
use iced::widget::svg::Handle;
use iced::alignment::Vertical;
use iced_font_awesome::{fa_icon, fa_icon_brands, fa_icon_solid};

use crate::{ICON, Message, NoctiG, Page};
use crate::formatting::{font, theme};

pub fn view(app: &NoctiG) -> Element<'_, Message> {
    container(row![
        // Left navigation sidebar
        container(column![
            view_branding(),

            space().height(32),

            button("Projects")
                .style(theme::button_primary)
                .on_press(Message::SwitchPage(Page::Home))
                .padding(12.0)
                .width(Length::Fill),
            button("Live")
                .style(theme::button_text)
                .on_press(Message::SwitchPage(Page::Live))
                .padding(12.0)
                .width(Length::Fill),
            button("Help")
                .style(theme::button_text)
                .on_press(Message::SwitchPage(Page::Help))
                .padding(12.0)
                .width(Length::Fill),

            space().height(Length::Fill),

            button(fa_icon_solid("gear").size(16.0))
                .style(theme::button_text)
                .on_press(Message::SwitchPage(Page::Settings))
                .padding(16.0)
        ].spacing(6.0)).width(Length::Fixed(256.0)).padding(12.0),

        // Main center content
        container(row![
            column![
                space().height(12.0),
                text("Projects").size(32.0),
                space().height(20.0),

                // Project search / action buttons
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
                    
                    button("New project")
                        .style(theme::button_primary)
                        .on_press(Message::CreateProjectWizard)
                        .padding([8.0, 12.0]),
                    button("Open existing")
                        .style(theme::button_secondary)
                        .on_press(Message::OpenProject)
                        .padding([8.0, 12.0])
                ].spacing(8.0),

                space().height(12.0),

                // Recent projects list
                container(
                    scrollable(
                        column![
                            space().height(2.0),
                            view_project(),
                            view_project(),
                            view_project(),
                            view_project(),
                            view_project(),
                            view_project(),
                            view_project(),
                            view_project(),
                            view_project(),
                            space().height(2.0),
                        ].padding([8.0, 12.0]),
                    )
                ).style(theme::container_recent_projects)
                    .width(Length::Fill)
                    .height(Length::Fill),

                space().height(12.0),

                // Informational footer links
                row![
                    button(text("Privacy Policy").size(12.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::SwitchPage(Page::Licenses))
                        .padding([4.0, 8.0]),

                    container(text("•")
                        .style(theme::text_secondary)
                        .size(14.0)
                    ).padding(Padding { left: 0.0, top: 3.0, right: 0.0, bottom: 0.0 }),

                    button(text("Licenses").size(12.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::SwitchPage(Page::Licenses))
                        .padding([4.0, 8.0]),

                    space().width(Length::Fill),

                    button(fa_icon_brands("github").color(theme::CLEAR_DARK_TEXT_SECONDARY).size(14.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::SwitchPage(Page::Licenses))
                        .padding(8.0)
                ]
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

fn view_project<'a>() -> Element<'a, Message>  {
    let path = "/home/user/Projects/Test.ngp";
    
    button(
        container(row![
            space().width(4.0),

            fa_icon("window-maximize").size(16.0),

            column![
                text("Test project").style(theme::text_primary),
                text(path).style(theme::text_secondary).size(12.0)    // TODO: Make this ellipsis in case of too small of available space
            ].spacing(1.0).width(Length::Fill).padding([0.0, 24.0]),

            text("2025/12/22 14:44").style(theme::text_tertiary).size(12.0),

            space().width(4.0),
        ].align_y(Vertical::Center).padding([12.0, 4.0]))
    )
        .on_press(Message::OpenProjectPath(path.to_string()))
        .style(theme::button_text_secondary)
        .into()
}