use iced::alignment::Vertical;
use iced::{Element, Length, Padding};
use iced::widget::{Row, button, center, column, container, row, scrollable, space, text, text_input};
use iced_font_awesome::fa_icon_solid;

use crate::{Message, ProjectConfiguration, formatting::theme};

pub fn view<'a>(project: &'a ProjectConfiguration) -> Element<'a, Message> {

    // Build tags
    let tags = Row::from_iter(
        project.tags.iter().enumerate().map(|(i, tag)| 
            container(
                row![
                    text(tag).size(14.0),
                    button(center("\u{00D7}").padding(Padding{top: -2.0, ..Default::default()}))
                        .on_press(Message::RemoveTag(i))
                        .width(Length::Fixed(28.0))
                        .height(Length::Fixed(28.0))
                        .padding(0)
                        .style(theme::button_secondary)
                ].spacing(4.0).align_y(Vertical::Center)
            ).padding(Padding { 
                left: 8.0, 
                top: 0.0, 
                right: 0.0, 
                bottom: 0.0 
            }).style(theme::container_tag).into()
        )
    );

    column![
        scrollable(
            column![
                space().height(20.0),

                column![
                    text("Project Name").size(14.0),

                    text_input("", &project.name)
                        .style(theme::text_input)
                        .on_input(Message::ProjectNameChanged)
                        .width(Length::Fill)
                        .padding([8.0, 12.0]),
                ].spacing(6.0),

                column![
                    text("Location").size(14.0),

                    row![
                        text_input("", &project.path)
                            .style(theme::text_input)
                            .on_input(Message::ProjectLocationChanged)
                            .width(Length::Fill)
                            .padding([8.0, 12.0]),

                        button("Browse")
                            .style(theme::button_secondary)
                            .on_press(Message::BrowseProjectLocation)
                            .padding([8.0, 12.0])
                    ].spacing(8.0)
                ].spacing(6.0),

                column![
                    text("Tags").size(14.0),

                    row![
                        text_input("", &project.new_tag)
                            .style(theme::text_input)
                            .on_input(Message::NewTagChanged)
                            .width(Length::Fill)
                            .padding([8.0, 12.0]),

                        button(fa_icon_solid("plus").size(14.0))
                            .style(theme::button_secondary)
                            .on_press(Message::AddTag)
                            .padding([12.0, 12.0])
                    ].spacing(8.0),

                    space(),

                    tags.spacing(8.0).wrap()
                ].spacing(6.0),
            ].spacing(28.0)
        ),

        space().height(Length::Fill)
    ].into()
}