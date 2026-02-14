use iced::{Element, Length};
use iced::widget::{column, space, text};

use crate::{Message, NoctiG};
use crate::formatting::{font, theme};

pub fn view(_app: &NoctiG) -> Element<'_, Message> {
    column![
        space().height(12.0),

        text("Work in progress").font(*font::REGULAR_BOLD).size(15.0),

        text("This section is not yet implemented. In the future a collection of application settings should be listed here").style(theme::text_secondary),

        space().height(Length::Fill)
    ].spacing(8.0).into()
}
