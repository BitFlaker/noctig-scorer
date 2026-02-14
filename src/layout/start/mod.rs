use iced::{Element, Length, Padding, Theme};
use iced::widget::{button, column, container, row, space, svg, text};
use iced::widget::svg::Handle;
use iced::alignment::Vertical;
use iced_font_awesome::{fa_icon, fa_icon_brands, fa_icon_solid};

use crate::{ICON, Message, NoctiG, Page};
use crate::formatting::{font, theme};

pub mod settings;
pub mod stream;
pub mod home;
pub mod help;

pub fn view(app: &NoctiG) -> Element<'_, Message> {
    container(row![
        // Left navigation sidebar
        container(column![
            view_branding(),

            space().height(32),

            button(row![
                fa_icon("window-maximize").size(15.0),
                text("Projects")
            ].align_y(Vertical::Center).spacing(12.0))
                .style(nav_button_theme(app, Page::Home))
                .on_press(Message::SwitchPage(Page::Home))
                .padding(12.0)
                .width(Length::Fill),

            button(row![
                fa_icon_solid("rss").size(15.0),
                text("Stream")
            ].align_y(Vertical::Center).spacing(12.0))
                .style(nav_button_theme(app, Page::Stream))
                .on_press(Message::SwitchPage(Page::Stream))
                .padding(12.0)
                .width(Length::Fill),

            button(row![
                fa_icon("circle-question").size(15.0),
                text("Help")
            ].align_y(Vertical::Center).spacing(12.0))
                .style(nav_button_theme(app, Page::Help))
                .on_press(Message::SwitchPage(Page::Help))
                .padding(12.0)
                .width(Length::Fill),

            space().height(Length::Fill),

            button(fa_icon_solid("gear").size(15.0))
                .style(nav_button_theme(app, Page::Settings))
                .on_press(Message::SwitchPage(Page::Settings))
                .padding(16.0)
        ].spacing(6.0)).width(Length::Fixed(256.0)).padding(12.0),

        // Main center content
        container(row![
            column![
                space().height(12.0),
                text(page_title(app)).size(32.0),
                space().height(20.0),

                // Main page content
                match app.current_page {
                    Page::Home => home::view(app),
                    Page::Stream => stream::view(app),
                    Page::Help => help::view(app),
                    Page::Settings => settings::view(app),
                    _ => panic!("Invalid page on start page")
                },

                space().height(12.0),

                // Informational footer links
                row![
                    button(text("Privacy Policy").size(12.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::ShowPrivacyPolicy)
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

                    button(fa_icon_brands("github").size(14.0))
                        .style(theme::button_text_secondary)
                        .on_press(Message::ShowSourceCode)
                        .padding(8.0)
                ]
            ],

            space().width(24.0)
        ]).width(Length::Fill).height(Length::Fill).padding(12.0)
    ]).into()
}

fn nav_button_theme<'a>(app: &NoctiG, page: Page) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    if app.current_page == page {
        theme::button_primary
    } else {
        theme::button_text
    }
}

fn page_title<'a>(app: &NoctiG) -> String {
    match app.current_page {
        Page::Home => "Projects",
        Page::Stream => "Stream",
        Page::Help => "Help",
        Page::Settings => "Settings",
        _ => "Unknown"
    }.to_string()
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
