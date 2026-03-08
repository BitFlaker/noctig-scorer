use iced::{Border, Color, Element, Font, Length, Point, Rectangle, Size, Theme};
use iced::widget::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::layout::{self, Limits};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{Text, Widget};
use iced::mouse::{self, Button};
use iced::border::Radius;
use std::collections::HashMap;
use std::cmp::Ordering;

use crate::Marker;

const MARKER_PADDING: f32 = 8.0;
const MARKER_WIDTH: f32 = 6.0;
const ANNOTATION_PADDING: f32 = 6.0;

pub struct AnnotationMarkData {
    pub marker: Marker,
    pub position: f32,
    pub duration: f32,
    pub text: String
}

pub struct SignalMarkers<'a, Message, Theme>
where
    Theme: Catalog
{
    signal_count: usize,
    is_global: bool,
    global_markers: Vec<(Marker, f32)>,
    local_markers: Vec<(u32, (Marker, f32))>,
    global_annotations: Vec<AnnotationMarkData>,
    local_annotations: Vec<(u32, AnnotationMarkData)>,
    current_marker: Marker,

    class: Theme::Class<'a>,
    width: Length,
    height: Length,
    on_marked: Option<Box<dyn Fn(Option<usize>, f32) -> Message>>,
    on_drag_marked: Option<Box<dyn Fn(Option<usize>, f32, f32) -> Message>>
}

impl<'a, Message, Theme> SignalMarkers<'a, Message, Theme>
where
    Theme: Catalog
{
    pub fn new(signal_count: usize) -> Self {
        Self {
            signal_count,
            is_global: false,
            global_markers: Vec::new(),
            local_markers: Vec::new(),
            global_annotations: Vec::new(),
            local_annotations: Vec::new(),
            current_marker: Marker::Red,

            class: Theme::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            on_marked: None,
            on_drag_marked: None
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, length: impl Into<Length>) -> Self {
        self.height = length.into();
        self
    }

    pub fn global(mut self, is_global: bool) -> Self {
        self.is_global = is_global;
        self
    }

    pub fn global_markers(mut self, mut markers: Vec<(Marker, f32)>) -> Self {
        markers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        self.global_markers = markers;
        self
    }

    pub fn local_markers(mut self, mut markers: Vec<(u32, (Marker, f32))>) -> Self {
        markers.sort_by(|a, b| a.1.1.partial_cmp(&b.1.1).unwrap_or(Ordering::Equal));
        self.local_markers = markers;
        self
    }

    pub fn global_annotations(mut self, mut annotations: Vec<AnnotationMarkData>) -> Self {
        annotations.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap_or(Ordering::Equal));
        self.global_annotations = annotations;
        self
    }

    pub fn local_annotations(mut self, mut annotations: Vec<(u32, AnnotationMarkData)>) -> Self {
        annotations.sort_by(|a, b| a.1.position.partial_cmp(&b.1.position).unwrap_or(Ordering::Equal));
        self.local_annotations = annotations;
        self
    }

    pub fn current_marker(mut self, current_marker: Marker) -> Self {
        self.current_marker = current_marker;
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    pub fn on_marked<F>(mut self, on_marked: F) -> Self
    where
        F: Fn(Option<usize>, f32) -> Message + 'static
    {
        self.on_marked = Some(Box::new(on_marked));
        self
    }

    pub fn on_drag_marked<F>(mut self, on_drag_marked: F) -> Self
    where
        F: Fn(Option<usize>, f32, f32) -> Message + 'static
    {
        self.on_drag_marked = Some(Box::new(on_drag_marked));
        self
    }
}

#[derive(Debug, Default)]
struct State {
    drag_state: Option<DragState>,
    hover_state: Option<HoverState>,
}

#[derive(Debug, Default)]
struct HoverState {
    signal_index: usize,
    position: Point<f32>,
}

#[derive(Debug, Default)]
struct DragState {
    start_signal_index: usize,
    start_position: Point<f32>,
    width: f32
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn markers<'a, Message, Theme>(signal_count: usize) -> SignalMarkers<'a, Message, Theme>
where
    Theme: Catalog
{
    SignalMarkers::new(signal_count)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for SignalMarkers<'a, Message, Theme>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn layout(
            &mut self,
            _tree: &mut Tree,
            _renderer: &Renderer,
            limits: &Limits,
        ) -> iced::advanced::layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
            &mut self,
            tree: &mut Tree,
            event: &iced::Event,
            layout: layout::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn iced::advanced::Clipboard,
            shell: &mut iced::advanced::Shell<'_, Message>,
            viewport: &iced::Rectangle,
        ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let signal_height = bounds.height / self.signal_count as f32;

        match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { position: _ }) => {
                // TODO: For the drag to still be updated when dragging outside the widgets area,
                //       the position from the event has to be used and transformed,
                //       as cursor.position() would be None

                if let Some(position) = cursor.position() && position.y - bounds.y <= signal_height * self.signal_count as f32 {
                    let new_position = Point::new(
                        (position.x - bounds.x) / bounds.width,
                        ((position.y - bounds.y) % signal_height) / signal_height
                    );
                    let last_position = state.hover_state.as_ref().map(|s| s.position).unwrap_or(new_position);

                    // Update the currently hovered signal and position
                    let signal_index = ((position.y - bounds.y) / signal_height) as usize;
                    state.hover_state = Some(HoverState {
                        signal_index,
                        position: new_position
                    });

                    // Update the drag width
                    if let Some(state) = &mut state.drag_state {
                        state.width += (new_position - last_position).x;
                    }

                    shell.request_redraw();
                }
                else if state.drag_state.is_none() {
                    let was_none = state.hover_state.is_none();
                    state.hover_state = None;
                    if !was_none {
                        shell.request_redraw();
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(position) = cursor.position() {
                    if viewport.contains(position) {
                        let signal_index = ((position.y - bounds.y) / signal_height) as usize;
                        let start_position = Point::new(
                            (position.x - bounds.x) / bounds.width,
                            ((position.y - bounds.y) % signal_height) / signal_height
                        );

                        state.drag_state = Some(DragState {
                            start_signal_index: signal_index,
                            start_position,
                            width: 0.0,
                        });
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if let Some(drag) = &state.drag_state {
                    if drag.width == 0.0 && let Some(hover) = &state.hover_state {  // It is a marker highlight
                        if let Some(on_marked) = &self.on_marked {
                            shell.publish(
                                on_marked(if self.is_global {
                                    None
                                } else {
                                    Some(hover.signal_index)
                                }, hover.position.x)
                            );
                        }
                    } else {    // It is an annotation (or marker with duration)
                        if let Some(on_drag_marked) = &self.on_drag_marked {
                            let start_offset = if drag.width < 0.0 { drag.width } else { 0.0 };
                            shell.publish(
                                on_drag_marked(if self.is_global {
                                    None
                                } else {
                                    Some(drag.start_signal_index)
                                }, drag.start_position.x + start_offset, drag.width.abs())
                            );
                        }
                    }

                    state.drag_state = None;
                    shell.request_redraw();
                }
            }
            _ => { }
        };
    }

    fn draw(
            &self,
            tree: &iced::advanced::widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            _style: &renderer::Style,
            layout: iced::advanced::Layout<'_>,
            _cursor: iced::advanced::mouse::Cursor,
            viewport: &iced::Rectangle,
        ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let signal_height = bounds.height / self.signal_count as f32;

        let style = theme.style(&self.class);

        // TODO: The text of annotations does not get hidden by overlapping markers, instead they appear
        //       always on top even when they should be (partially) hidden

        // Draw all global annotations
        for data in &self.global_annotations {
            let Some(color) = style.marker_colors.get(&data.marker) else {
                continue;
            };
            let color_highlight = Color::from_rgb(
                color.r * 0.48,
                color.g * 0.48,
                color.b * 0.48
            );
            let quad_block = get_annotation_quad(
                0,
                true,
                data.position,
                data.duration,
                bounds,
                signal_height
            );
            let quad_highlight = generate_annotation_highlight(&quad_block.bounds);

            renderer.fill_quad(quad_block, color.clone());
            renderer.fill_quad(quad_highlight, color_highlight);
            renderer.fill_text(
                generate_annotation_text(&data.text, quad_highlight.bounds.size()),
                Point::new(quad_highlight.bounds.x + ANNOTATION_PADDING, quad_highlight.bounds.center_y()),
                color.clone(),
                *viewport,
            );
        }

        // Draw all local annotations
        for (signal_index, data) in &self.local_annotations {
            let Some(color) = style.marker_colors.get(&data.marker) else {
                continue;
            };
            let color_highlight = Color::from_rgb(
                color.r * 0.48,
                color.g * 0.48,
                color.b * 0.48
            );
            let quad_block = get_annotation_quad(
                *signal_index,
                false,
                data.position,
                data.duration,
                bounds,
                signal_height
            );
            let quad_highlight = generate_annotation_highlight(&quad_block.bounds);

            renderer.fill_quad(quad_block, color.clone());
            renderer.fill_quad(quad_highlight, color_highlight);
            renderer.fill_text(
                generate_annotation_text(&data.text, quad_highlight.bounds.size()),
                Point::new(quad_highlight.bounds.x + ANNOTATION_PADDING, quad_highlight.bounds.center_y()),
                color.clone(),
                *viewport,
            );
        }

        // Draw all global markers
        for (marker, position) in &self.global_markers {
            let Some(color) = style.marker_colors.get(marker) else {
                continue;
            };
            renderer.fill_quad(
                get_marker_quad(
                    0,
                    true,
                    *position,
                    bounds,
                    signal_height
                ),
                color.clone()
            );
        }

        // Draw all local markers
        for (signal_index, (marker, position)) in &self.local_markers {
            let Some(color) = style.marker_colors.get(marker) else {
                continue;
            };
            renderer.fill_quad(
                get_marker_quad(
                    *signal_index,
                    false,
                    *position,
                    bounds,
                    signal_height
                ),
                color.clone()
            );
        }

        // Get highlight marker color
        let color = style.marker_colors.get(&self.current_marker)
            .cloned()
            .unwrap_or(Color::WHITE)
            .scale_alpha(0.32);

        // Draw phantom highlights
        if let Some(drag) = &state.drag_state {             // Draw phantom drag highlight
            renderer.fill_quad(
                get_annotation_quad(
                    drag.start_signal_index as u32,
                    self.is_global,
                    drag.start_position.x,
                    drag.width,
                    bounds,
                    signal_height
                ),
                color
            );
        } else if let Some(hover) = &state.hover_state {    // Draw phantom marker highlight
            renderer.fill_quad(
                get_marker_quad(
                    hover.signal_index as u32,
                    self.is_global,
                    hover.position.x,
                    bounds,
                    signal_height
                ),
                color
            );
        };
    }
}

impl<'a, Message, Theme, Renderer> From<SignalMarkers<'a, Message, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Theme: Catalog + 'a,
    Message: 'a
{
    fn from(markers: SignalMarkers<'a, Message, Theme>) -> Self {
        Self::new(markers)
    }
}

fn generate_annotation_highlight(bounds: &Rectangle<f32>) -> Quad {
    let mut bounds = bounds.clone();
    bounds.height = 24.0;
    let border = Border::default().rounded(Radius {
        top_left: 3.0,
        top_right: 3.0,
        ..Default::default()
    });

    Quad {
        bounds,
        border,
        ..Default::default()
    }
}

fn generate_annotation_text(text: &String, mut text_bounds: Size<f32>) -> Text {
    text_bounds.width -= 2.0 * ANNOTATION_PADDING;

    Text {
        content: text.clone(),
        bounds: text_bounds,
        align_x: Alignment::Left,
        align_y: iced::alignment::Vertical::Center,
        line_height: LineHeight::Relative(12.0.into()),
        shaping: Shaping::Basic,
        size: 12.0.into(),
        font: Font::DEFAULT,
        wrapping: Wrapping::None,
    }
}

fn get_annotation_quad(
    signal_index: u32,
    is_global: bool,
    start_position: f32,
    duration: f32,
    bounds: Rectangle,
    signal_height: f32
) -> Quad {
    let duration_width = duration * bounds.width;
    let y_start = signal_index as f32 * if is_global { 0.0 } else { signal_height };
    let start_offset = if duration_width < 0.0 { duration_width } else { 0.0 };
    let marker_height = if is_global { bounds.height } else { signal_height };

    Quad {
        bounds: Rectangle::new(
            Point::new(start_position * bounds.width - MARKER_WIDTH / 2.0 + start_offset, bounds.y + y_start + MARKER_PADDING),
            Size::new(MARKER_WIDTH + duration_width.abs(), marker_height - 2.0 * MARKER_PADDING)
        ),
        border: Border::default().rounded(3.0),
        ..Default::default()
    }
}

fn get_marker_quad(
    signal_index: u32,
    is_global: bool,
    position: f32,
    bounds: Rectangle,
    signal_height: f32
) -> Quad {
    let y_start = signal_index as f32 * if is_global { 0.0 } else { signal_height };
    let marker_height = if is_global { bounds.height } else { signal_height };

    Quad {
        bounds: Rectangle::new(
            Point::new(position * bounds.width - MARKER_WIDTH / 2.0, bounds.y + y_start + MARKER_PADDING),
            Size::new(MARKER_WIDTH, marker_height - 2.0 * MARKER_PADDING)
        ),
        border: Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

//#########################//
//         STYLING         //
//#########################//

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub marker_colors: HashMap<Marker, Color>
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(_theme: &Theme) -> Style {
    let mut marker_colors = HashMap::new();
    marker_colors.insert(Marker::Red, Color::from_rgb8(255, 0, 0));
    marker_colors.insert(Marker::Orange, Color::from_rgb8(255, 128, 0));
    marker_colors.insert(Marker::Yellow, Color::from_rgb8(255, 255, 0));
    marker_colors.insert(Marker::Green, Color::from_rgb8(0, 255, 0));
    marker_colors.insert(Marker::Cyan, Color::from_rgb8(0, 128, 128));
    marker_colors.insert(Marker::Blue, Color::from_rgb8(0, 0, 255));
    marker_colors.insert(Marker::Purple, Color::from_rgb8(128, 0, 128));

    Style {
        marker_colors
    }
}
