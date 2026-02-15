use iced::widget::{Column, button, column, container, row, scrollable, space, text, tooltip};
use iced::widget::tooltip::Position;
use iced::alignment::Vertical;
use iced::{Element, Length, Padding};
use iced_font_awesome::fa_icon;

use crate::views::collapsible::Collapsible;
use crate::{LicenseData, Message, NoctiG};
use crate::formatting::{font, theme};

pub fn view(app: &NoctiG) -> Element<'_, Message> {
    column![
        column![
            text("Licenses").size(32.0),

            text("The section below lists the projects, libraries, etc. used in this application. Every entry includes a link to the project homepage / source code alongside the license text").style(theme::text_secondary),

            container(space())
                .height(1.0)
                .width(Length::Fill)
                .style(theme::stroke_primary),
        ].spacing(16.0).padding(Padding {
            left: 24.0,
            top: 20.0,
            right: 24.0,
            bottom: 0.0
        }),

        container(
            scrollable(
                column![
                    space().height(20.0),

                    row![
                        text("Ported").font(*font::REGULAR_BOLD).size(16.0),
                        tooltip(
                            fa_icon("circle-question").style(theme::text_secondary).size(14.0),
                            container(
                                text("Projects which have been (partially) ported to Rust and modified to integrate into this application").size(13.0).center(),
                            ).padding([8.0, 12.0]).style(theme::container_tooltip).max_width(192.0),
                            Position::Bottom
                        )
                    ].spacing(8.0).align_y(Vertical::Center),

                    space().height(8.0),
                    Column::from_iter(app.licenses[0]
                        .iter()
                        .enumerate()
                        .map(|(i, c)| c.view(
                            |e| Message::ToggleExpandLicense(0, i, e),
                            view_license_body
                        ))
                    ).spacing(6.0),
                    space().height(16.0),

                    row![
                        text("Libraries").font(*font::REGULAR_BOLD).size(16.0),
                        tooltip(
                            fa_icon("circle-question").style(theme::text_secondary).size(14.0),
                            container(
                                text("Libraries this application directly depends on. For transitive dependencies, see the homepage of the individual libraries").size(13.0).center(),
                            ).padding([8.0, 12.0]).style(theme::container_tooltip).max_width(192.0),
                            Position::Bottom
                        )
                    ].spacing(8.0).align_y(Vertical::Center),

                    space().height(8.0),
                    Column::from_iter(app.licenses[1]
                        .iter()
                        .enumerate()
                        .map(|(i, c)| c.view(
                            |e| Message::ToggleExpandLicense(1, i, e),
                            view_license_body
                        ))
                    ).spacing(6.0),
                    space().height(16.0),

                    row![
                        text("Other").font(*font::REGULAR_BOLD).size(16.0),
                        tooltip(
                            fa_icon("circle-question").style(theme::text_secondary).size(14.0),
                            container(
                                text("All other projects (used for e.g. fonts)").size(13.0).center(),
                            ).padding([8.0, 12.0]).style(theme::container_tooltip).max_width(192.0),
                            Position::Bottom
                        )
                    ].spacing(8.0).align_y(Vertical::Center),

                    space().height(8.0),
                    Column::from_iter(app.licenses[2]
                        .iter()
                        .enumerate()
                        .map(|(i, c)| c.view(
                            |e| Message::ToggleExpandLicense(2, i, e),
                            view_license_body
                        ))
                    ).spacing(6.0),
                    space().height(24.0),
                ].spacing(4.0).padding(Padding {
                    left: 12.0,
                    top: 0.0,
                    right: 36.0,
                    bottom: 0.0
                })
            ).width(Length::Fill).height(Length::Fill)
        ).width(Length::Fill).height(Length::Fill).padding(Padding {
            left: 24.0,
            ..Default::default()
        })
    ].into()
}

fn view_license_body<'a>(license: &Collapsible<LicenseData>) -> Element<'a, Message> {
    let data = license.content_data();

    column![
        column![
            row![
                text("License:").size(13.0).style(theme::text_primary),
                text(license.subtitle().clone()).size(13.0).style(theme::text_primary),
            ].align_y(Vertical::Center).spacing(4.0),
            row![
                text("Homepage:").size(13.0).style(theme::text_primary),
                hyperlink(data.url.clone()),
            ].align_y(Vertical::Center).spacing(4.0),
        ].padding([0.0, 12.0]),

        space().height(8.0),
        container(space())
            .height(1.0)
            .width(Length::Fill)
            .style(theme::stroke_primary),
        space().height(16.0),

        container(text(data.license_texts.join("\n\n----------------------------------------\n\n"))
            .style(theme::text_secondary)
        ).padding([0.0, 12.0])
    ].into()
}

fn hyperlink<'a>(url: String) -> Element<'a, Message> {
    button(text(url.clone()).size(13.0).style(theme::text_url))
        .style(theme::button_text_secondary)
        .on_press(Message::OpenURL(url))
        .padding([4.0, 8.0])
        .into()
}

const ICONS: [Option<&str>; 2] = [
    None,
    Some("scale-balanced"),
];

pub fn load_licenses() -> [Vec<Collapsible<LicenseData>>; 3] {
    [
        // Ported
        vec![
            Collapsible::<LicenseData>::new("lspopt", "MIT license", LicenseData::new(
                "https://github.com/hbldh/lspopt",
                &[include_str!("../external/lspopt/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("Scientific colour maps", "MIT license", LicenseData::new(
                "https://doi.org/10.5281/zenodo.1243862",
                &[include_str!("../external/scientific_colour_maps/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("SciPy", "BSD-3-Clause license", LicenseData::new(
                "https://github.com/scipy/scipy",
                &[include_str!("../external/scipy/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("Yasa", "BSD-3-Clause license", LicenseData::new(
                "https://github.com/raphaelvallat/yasa",
                &[include_str!("../external/yasa/LICENSE")]
            ), ICONS.clone()),
        ],

        // Libraries
        vec![
            Collapsible::<LicenseData>::new("iced", "MIT license", LicenseData::new(
                "https://github.com/iced-rs/iced",
                &[include_str!("../external/_libraries/iced/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("iced_font_awesome", "MIT license", LicenseData::new(
                "https://github.com/danielmbomfim/iced_font_awesome",
                &[include_str!("../external/_libraries/iced_font_awesome/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("bytemuck", "MIT / Zlib / Apache 2.0 licenses", LicenseData::new(
                "https://github.com/Lokathor/bytemuck",
                &[
                    include_str!("../external/_libraries/bytemuck/LICENSE-MIT"),
                    include_str!("../external/_libraries/bytemuck/LICENSE-ZLIB"),
                    include_str!("../external/_libraries/bytemuck/LICENSE-APACHE"),
                ]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("ndarray", "MIT license", LicenseData::new(
                "https://github.com/rust-ndarray/ndarray",
                &[include_str!("../external/_libraries/ndarray/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("ndarray-npy", "MIT license", LicenseData::new(
                "https://github.com/jturner314/ndarray-npy",
                &[include_str!("../external/_libraries/ndarray-npy/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("rayon", "MIT / Apache 2.0 licenses", LicenseData::new(
                "https://github.com/rayon-rs/rayon",
                &[
                    include_str!("../external/_libraries/rayon/LICENSE-MIT"),
                    include_str!("../external/_libraries/rayon/LICENSE-APACHE")
                ]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("realfft", "MIT license", LicenseData::new(
                "https://github.com/HEnquist/realfft",
                &[include_str!("../external/_libraries/realfft/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("log", "MIT license", LicenseData::new(
                "https://github.com/rust-lang/log",
                &[include_str!("../external/_libraries/log/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("env_logger", "MIT / Apache 2.0 licenses", LicenseData::new(
                "https://github.com/rust-cli/env_logger",
                &[
                    include_str!("../external/_libraries/env_logger/LICENSE-MIT"),
                    include_str!("../external/_libraries/env_logger/LICENSE-APACHE")
                ]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("edf-rs", "MIT license", LicenseData::new(
                "https://github.com/BitFlaker/edf-rs",
                &[include_str!("../external/_libraries/edf-rs/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("rfd", "MIT license", LicenseData::new(
                "https://github.com/PolyMeilex/rfd",
                &[include_str!("../external/_libraries/rfd/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("xml", "MIT license", LicenseData::new(
                "https://github.com/kornelski/xml-rs",
                &[include_str!("../external/_libraries/xml/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("serde-xml-rs", "MIT license", LicenseData::new(
                "https://github.com/RReverser/serde-xml-rs",
                &[include_str!("../external/_libraries/serde-xml-rs/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("serde_json", "MIT license", LicenseData::new(
                "https://github.com/serde-rs/json",
                &[include_str!("../external/_libraries/serde_json/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("serde", "MIT license", LicenseData::new(
                "https://github.com/serde-rs/serde",
                &[include_str!("../external/_libraries/serde/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("rusqlite", "MIT license", LicenseData::new(
                "https://github.com/rusqlite/rusqlite",
                &[include_str!("../external/_libraries/rusqlite/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("SQLite", "Public Domain", LicenseData::new(
                "https://www.sqlite.org/copyright.html",
                &[]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("chrono", "MIT / Apache 2.0 licenses", LicenseData::new(
                "https://github.com/chronotope/chrono",
                &[
                    include_str!("../external/_libraries/chrono/LICENSE-MIT"),
                    include_str!("../external/_libraries/chrono/LICENSE-APACHE")
                ]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("itertools", "MIT license", LicenseData::new(
                "https://github.com/rust-itertools/itertools",
                &[include_str!("../external/_libraries/itertools/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("webbrowser", "MIT license", LicenseData::new(
                "github.com/amodm/webbrowser-rs",
                &[include_str!("../external/_libraries/webbrowser/LICENSE")]
            ), ICONS.clone()),
            Collapsible::<LicenseData>::new("md-5", "MIT license", LicenseData::new(
                "https://github.com/RustCrypto/hashes",
                &[include_str!("../external/_libraries/md-5/LICENSE")]
            ), ICONS.clone()),
        ],

        // Other
        vec![
            Collapsible::<LicenseData>::new("Font Awesome", "CC BY 4.0 / SIL OFL 1.1", LicenseData::new(
                "https://fontawesome.com",
                &[include_str!("../external/_other/FontAwesome/LICENSE")]
            ), ICONS.clone())
        ]
    ]
}
