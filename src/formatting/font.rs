use iced::Font;
use iced::font::Weight;
use std::sync::LazyLock;

pub const REGULAR_BOLD: LazyLock<Font> = LazyLock::new(|| {
    let mut font = Font::default();
    font.weight = Weight::Bold;
    font
});