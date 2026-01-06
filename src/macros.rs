#[macro_export]
macro_rules! key_legend {
    ([$($keys:expr),*], $description:expr) => {
        #[allow(unstable_name_collisions)]  // Until `intersperse_with` is stabilized, then remove itertools
        row![
            iced::widget::Row::from_iter([$($keys),*].iter().map(|key| 
                container(
                    text(*key).size(11.0)
                ).width(24.0).align_x(Alignment::Center).padding([4.0, 0.0]).style(theme::container_key).into()
            ).intersperse_with(|| text("/").style(theme::text_secondary).into())).spacing(4.0).align_y(Vertical::Center),
            text($description).size(14.0).style(theme::text_secondary)
        ].spacing(6.0).align_y(Vertical::Center)
    };
    ($key:expr, $description:expr) => {
        row![
            container(
                text($key).size(11.0)
            ).width(24.0).align_x(Alignment::Center).padding([4.0, 0.0]).style(theme::container_key),
            text($description).size(14.0).style(theme::text_secondary)
        ].spacing(6.0).align_y(Vertical::Center)
    };
}