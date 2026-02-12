use iced::border;
use iced::border::Radius;
use iced::theme::palette::Danger;
use iced::theme::palette::Primary;
use iced::theme::palette::Secondary;
use iced::theme::palette::Success;
use iced::theme::palette::Warning;
use iced::theme::Palette;
use iced::theme::palette;
use iced::widget;
use iced::Theme;
use palette::Pair;
use iced::theme::palette::Background;
use iced::Color;

use crate::Stage;

pub const SPECTROGRAM_BORDER_WIDTH: f32 = 2.0;

// **************
// Color Palettes
// **************

#[macro_export]
macro_rules! rgba8 {
    ($r:expr, $g:expr, $b:expr) => {
        Color::from_rgb8($r, $g, $b)
    };
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        { Color::from_rgba8($r, $g, $b, $a) }
    };
}

pub const CLEAR_DARK_TEXT_SECONDARY: Color = rgba8!(130, 130, 130);
pub const CLEAR_DARK_TEXT_TERTIARY: Color = rgba8!(80, 80, 80);

pub const CLEAR_DARK: Palette = Palette {
    // background: rgba8!( 23,  24,  27),
    background: rgba8!( 28,  28,  28),
    text:       rgba8!(230, 230, 230),
    primary:    rgba8!( 18, 124, 218),
    success:    rgba8!( 18, 102,  79),
    danger:     rgba8!(195,  66,  63),
    warning:    rgba8!(195,  66,  63),
};

pub const CLEAR_DARK_STAGES: StagePalette = StagePalette {
    w:     rgba8!(213, 150,  68),   // rgba8!(80, 170,  95)
    r:     rgba8!(215,  74,  47),
    n1:    rgba8!( 80, 165, 255),
    n2:    rgba8!(130, 105, 235),
    n3:    rgba8!(180,  75, 215),
    unset: Color::TRANSPARENT
};

pub const CLEAR_LIGHT_STAGES: StagePalette = StagePalette {
    w:     rgba8!(213, 150,  68),
    r:     rgba8!(196,  60,  33),
    n1:    rgba8!( 44,  62,  80),
    n2:    rgba8!(161,  54, 212),
    n3:    rgba8!(134, 171, 213),
    unset: Color::TRANSPARENT
};

pub fn generate_extended(palette: Palette) -> palette::Extended {
    palette::Extended {
        background: Background {
            base:      Pair::new(CLEAR_DARK.background, palette.text),
            weakest:   Pair::new(rgba8!( 22,  22,  22), palette.text),
            weaker:    Pair::new(rgba8!( 44,  44,  44), palette.text),
            weak:      Pair::new(rgba8!( 50,  50,  50), palette.text),
            neutral:   Pair::new(rgba8!( 56,  56,  56), palette.text),
            strong:    Pair::new(rgba8!( 62,  62,  62), palette.text),
            stronger:  Pair::new(rgba8!( 68,  68,  68), palette.text),
            strongest: Pair::new(rgba8!( 74,  74,  74), palette.text)
        },
        primary: Primary::generate(
            palette.primary,
            palette.background,
            palette.text,
        ),
        secondary: Secondary {
            weak: Pair::new(rgba8!( 38,  38,  38), palette.text),
            base: Pair::new(rgba8!( 50,  50,  50), palette.text),
            strong: Pair::new(rgba8!( 62,  62,  62), palette.text)
        },
        success: Success::generate(
            palette.success,
            palette.background,
            palette.text,
        ),
        danger: Danger::generate(
            palette.danger,
            palette.background,
            palette.text,
        ),
        warning: Warning::generate(
            palette.warning,
            palette.background,
            palette.text,
        ),
        is_dark: true,
    }
}

pub fn get_log_palette(theme: &Theme) -> StagePalette {
    match theme.palette() {
        CLEAR_DARK => CLEAR_DARK_STAGES,
        _ => CLEAR_LIGHT_STAGES
    }
}

pub fn container_stage_r(theme: &Theme) -> widget::container::Style {
    let palette = get_log_palette(theme);
    container_stage_base(palette.r)
}

pub fn container_stage_w(theme: &Theme) -> widget::container::Style {
    let palette = get_log_palette(theme);
    container_stage_base(palette.w)
}

pub fn container_stage_n1(theme: &Theme) -> widget::container::Style {
    let palette = get_log_palette(theme);
    container_stage_base(palette.n1)
}

pub fn container_stage_n2(theme: &Theme) -> widget::container::Style {
    let palette = get_log_palette(theme);
    container_stage_base(palette.n2)
}

pub fn container_stage_n3(theme: &Theme) -> widget::container::Style {
    let palette = get_log_palette(theme);
    container_stage_base(palette.n3)
}

pub fn container_stage_unset(theme: &Theme) -> widget::container::Style {
    let palette = get_log_palette(theme);
    container_stage_base(palette.unset)
}

fn container_stage_base(color: Color) -> widget::container::Style {
    widget::container::Style {
        background: Some(color.into()),
        border: border::Border {
            width: 1.0,
            radius: 4.0.into(),
            color,
        },
        ..widget::container::Style::default()
    }
}

pub fn container_loading_spectrogram(theme: &Theme) -> widget::container::Style {
    let color = theme.extended_palette().background.weak.color;
    widget::container::Style {
        background: Some(color.into()),
        border: border::Border {
            radius: Radius {
                top_left: 8.0,
                bottom_left: 8.0,
                ..Default::default()
            },
            ..Default::default()
        },
        ..widget::container::Style::default()
    }
}

pub fn container_spectrogram(theme: &Theme) -> widget::container::Style {
    let color = theme.extended_palette().background.weak.color;
    widget::container::Style {
        background: None,
        border: border::Border {
            width: SPECTROGRAM_BORDER_WIDTH,
            radius: 8.0.into(),
            color,
        },
        ..widget::container::Style::default()
    }
}

pub fn container_tag(theme: &Theme) -> widget::container::Style {
    let stroke = theme.extended_palette().background.weak.color;
    let mut style = container_stage_base(stroke);
    style.border.radius = 10.0.into();

    style
}

pub fn container_key(theme: &Theme) -> widget::container::Style {
    let color_background = theme.extended_palette().background.stronger.color;
    let color_border = theme.extended_palette().background.strong.color;

    widget::container::Style {
        background: Some(color_background.into()),
        border: border::Border {
            width: 1.0,
            radius: 4.0.into(),
            color: color_border,
        },
        ..widget::container::Style::default()
    }
}

pub fn container_recent_projects(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();

    widget::container::Style {
        background: Some(palette.background.weakest.color.into()),
        border: border::Border {
            width: 1.0,
            radius: 10.0.into(),
            color: palette.background.neutral.color,
        },
        ..widget::container::Style::default()
    }
}

pub fn container_secondary(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();

    widget::container::Style {
        background: Some(palette.secondary.weak.color.into()),
        border: iced::Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn container_counter(theme: &Theme, is_current: bool) -> widget::container::Style {
    let palette = theme.palette();

    let color_current = palette.text;
    let color_other = Color::TRANSPARENT;

    widget::container::Style {
        background: Some(if is_current { color_current } else { color_other }.into()),
        border: border::Border {
            width: if is_current { 0.0 } else { 1.0 },
            radius: 999.0.into(),
            color: CLEAR_DARK_TEXT_TERTIARY,
            ..Default::default()
        },
        ..widget::container::Style::default()
    }
}

pub fn text_counter(theme: &Theme, is_current: bool) -> widget::text::Style {
    let palette = theme.extended_palette();

    let color_text_current = palette.background.weakest.color;
    let color_text_other = CLEAR_DARK_TEXT_SECONDARY;

    widget::text::Style {
        color: Some(if is_current { color_text_current } else { color_text_other }.into()),
    }
}

pub fn checkbox(theme: &Theme, status: iced::widget::checkbox::Status) -> widget::checkbox::Style {
    let style = iced::widget::checkbox::primary(theme, status);

    widget::checkbox::Style {
        border: border::Border {
            width: 1.0,
            radius: 5.0.into(),
            ..style.border
        },
        ..style
    }
}

pub fn button_current_create_page(theme: &Theme, status: iced::widget::button::Status, is_current: bool) -> widget::button::Style {
    let palette = theme.palette();
    let mut style = button_text_secondary(theme, status);

    let color_text_other = CLEAR_DARK_TEXT_SECONDARY;
    let color_text_current = palette.text;

    style.text_color = if is_current { color_text_current } else { color_text_other }.into();

    style
}

pub fn button_secondary(theme: &Theme, status: iced::widget::button::Status) -> widget::button::Style {
    use iced::widget::button::secondary;

    let mut style = secondary(theme, status);
    style.border.radius = 10.0.into();

    style
}

pub fn button_primary(theme: &Theme, status: iced::widget::button::Status) -> widget::button::Style {
    use iced::widget::button::primary;

    let mut style = primary(theme, status);
    style.border.radius = 10.0.into();

    style
}

pub fn button_text(theme: &Theme, status: iced::widget::button::Status) -> widget::button::Style {
    use iced::widget::button::text;
    use iced::widget::button::Status;

    let palette = theme.extended_palette();
    let color = palette.background.neutral.color;
    let text_color = palette.background.neutral.text;
    let mut style = text(theme, status);
    style.border.radius = 10.0.into();
    style.text_color = text_color.into();

    match status {
        Status::Pressed => {
            style.background = Some(color.scale_alpha(0.6).into());
        },
        Status::Hovered => {
            style.background = Some(color.scale_alpha(0.4).into());
        },
        _ => { },
    }

    style
}

pub fn button_text_secondary(theme: &Theme, status: iced::widget::button::Status) -> widget::button::Style {
    use iced::widget::button::Status;

    let mut style = button_text(theme, status);

    match status {
        Status::Pressed | Status::Hovered => {
            style.text_color = style.text_color.scale_alpha(1.0);
        },
        _ => {
            style.text_color = CLEAR_DARK_TEXT_SECONDARY;
        }
    }

    style
}

pub fn text_input(theme: &Theme, status: iced::widget::text_input::Status) -> widget::text_input::Style {
    use iced::widget::text_input::default;
    use iced::widget::text_input::Status;

    let palette = theme.extended_palette();
    let color = palette.background.weaker.color;
    let text_color = palette.background.strongest.text;

    let mut style = default(theme, status);
    style.border.radius = 10.0.into();
    style.background = color.into();
    style.border.color = color.into();
    style.value = text_color.into();
    style.placeholder = CLEAR_DARK_TEXT_SECONDARY.into();

    match status {
        Status::Hovered => {
            style.border.color = CLEAR_DARK_TEXT_SECONDARY.into();
        },
        Status::Focused { .. } => {
            style.border.color = text_color.into();
        },
        _ => { }
    }

    style
}

pub fn text_input_secondary(theme: &Theme, status: iced::widget::text_input::Status) -> widget::text_input::Style {
    let palette = theme.extended_palette();
    let mut style = text_input(theme, status);
    style.background = Color::TRANSPARENT.into();
    style.border.color = palette.background.neutral.color;

    style
}

pub struct StagePalette {
    pub r: Color,
    pub w: Color,
    pub n1: Color,
    pub n2: Color,
    pub n3: Color,
    pub unset: Color,
}

// **************
// Control styles
// **************

pub fn status_bar(theme: &Theme) -> widget::container::Style {
    let background_color = theme.extended_palette().background.weaker.color;

    widget::container::Style {
        background: Some(background_color.into()),
        ..widget::container::Style::default()
    }
}

pub fn stroke(theme: &Theme) -> widget::container::Style {
    let mut border_color = theme.extended_palette().background.strongest.color;
    border_color.a = 0.32;

    widget::container::Style {
        background: Some(border_color.into()),
        border: border::Border {
            width: 0.0,
            radius: 0.0.into(),
            color: border_color,
        },
        ..widget::container::Style::default()
    }
}

pub fn text_primary(theme: &Theme) -> widget::text::Style {
    widget::text::Style {
        color: Some(theme.palette().text),
    }
}

/// Text conveying some secondary information, like a footnote.
pub fn text_secondary(_theme: &Theme) -> widget::text::Style {
    let color = CLEAR_DARK_TEXT_SECONDARY;
    widget::text::Style {
        color: Some(color),
    }
}

pub fn text_tertiary(_theme: &Theme) -> widget::text::Style {
    let color = CLEAR_DARK_TEXT_TERTIARY;
    widget::text::Style {
        color: Some(color),
    }
}

pub fn border_background_base(theme: &Theme, stage: &Stage) -> widget::container::Style {
    match stage {
        Stage::W => container_stage_w(theme),
        Stage::R => container_stage_r(theme),
        Stage::N1 => container_stage_n1(theme),
        Stage::N2 => container_stage_n2(theme),
        Stage::N3 => container_stage_n3(theme),
        Stage::Unset => container_stage_unset(theme)//container_stage_transparent(),
    }
}

/// Text conveying some secondary information, like a footnote.
pub fn text_foreground_base(theme: &Theme, stage: &Stage) -> widget::text::Style {
    widget::text::Style {
        color: Some(if *stage == Stage::Unset {
            Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 }
        } else {
            theme.extended_palette().background.base.color
        }),
    }
}
