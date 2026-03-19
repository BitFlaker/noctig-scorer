use iced::{Border, Color, Element, Event, Length, Pixels, Rectangle, Size, Theme};
use iced::advanced::{Clipboard, Shell, mouse, renderer};
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::widget::tree::State;
use iced::advanced::renderer::Quad;
use iced::mouse::{Button, Cursor, Interaction};

use crate::Marker;
use crate::views::signal_markers;

const MARKER_POINTER_WIDTH_TOP: f32 = 4.0;
const MARKER_POINTER_WIDTH_BOTTOM: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct Highlight {
    pub time_ns: u64,
    pub duration: Option<u64>,
    pub marker: Marker,
}

pub struct TimelineScrollbar<'a, Message, Theme>
where
    Theme: Catalog + 'a
{
    total_ns: u64,
    window_width_ns: u64,
    zoom_factor: f32,
    highlights: Vec<Highlight>,

    top_height: f32,
    bottom_height: f32,

    pending_update_position: Option<(u64, u64)>,

    class: Theme::Class<'a>,
    on_viewport_change: Option<Box<dyn Fn(u64, u64) -> Message + 'a>>,
    on_zoom_change: Option<Box<dyn Fn(u64, u64) -> Message + 'a>>,
    on_position_set: Option<Box<dyn Fn(u64, u64) -> Message + 'a>>,
}

impl<'a, Message, Theme> TimelineScrollbar<'a, Message, Theme>
where
    Theme: Catalog + 'a
{
    pub fn new(total_ns: u64, window_width_ns: u64, highlights: Vec<Highlight>) -> Self {
        Self {
            total_ns,
            window_width_ns,
            zoom_factor: 6.0,
            highlights,
            top_height: 32.0,
            bottom_height: 16.0,
            pending_update_position: None,
            class: Theme::default(),
            on_viewport_change: None,
            on_zoom_change: None,
            on_position_set: None,
        }
    }

    /// Set the height of the top strip.
    pub fn top_height(mut self, height: impl Into<Pixels>) -> Self {
        self.top_height = height.into().0;
        self
    }

    /// Set the height of the bottom strip.
    pub fn bottom_height(mut self, height: impl Into<Pixels>) -> Self {
        self.bottom_height = height.into().0;
        self
    }

    /// Set the zoom factor of the top strip.
    pub fn zoom_factor(mut self, factor: f32) -> Self {
        self.zoom_factor = factor;
        self
    }

    /// Update the viewport position state. When `None`, the position will remain unchanged.
    pub fn update_position(mut self, bounds: Option<(u64, u64)>) -> Self {
        self.pending_update_position = bounds;
        self
    }

    /// Callback invoked when the the viewport thumb was moved.
    pub fn on_viewport_change(mut self, f: impl Fn(u64, u64) -> Message + 'a) -> Self {
        self.on_viewport_change = Some(Box::new(f));
        self
    }

    /// Callback invoked when the zoom-window thumb was moved.
    pub fn on_zoom_change(mut self, f: impl Fn(u64, u64) -> Message + 'a) -> Self {
        self.on_zoom_change = Some(Box::new(f));
        self
    }

    /// Callback invoked when the position changed and the thumbs are not being dragged anymore.
    /// The values passed to the callback are the start and end nanoseconds of the viewport.
    /// The values are equal to the values passed to the last [`on_viewport_change`] callback.
    pub fn on_position_set(mut self, f: impl Fn(u64, u64) -> Message + 'a) -> Self {
        self.on_position_set = Some(Box::new(f));
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    fn total_height(&self) -> f32 {
        self.top_height + self.bottom_height
    }

    fn zoom_width(&self) -> u64 {
        (self.total_ns as f64 / self.zoom_factor as f64) as u64
    }
}

fn ns_to_x(ns: u64, range_start: u64, range_end: u64, rect: Rectangle) -> f32 {
    if ns == 0 {
        return 0.0;
    }
    let t = (ns as i128 - range_start as i128) as f32 / (range_end - range_start) as f32;
    rect.x + t * rect.width
}

fn x_to_ns(x: f32, range_start: u64, range_end: u64, rect: Rectangle) -> u64 {
    if x == 0.0 {
        return range_start;
    }
    let t = (x - rect.x) / rect.width;
    let offset = t * (range_end - range_start) as f32;
    (range_start as i128 + offset as i128).max(0) as u64
}

#[derive(Debug, Default, Clone)]
pub struct DragState {
    target: DragTarget,
    grab_offset_x: f32,
    thumb_width: f32
}

#[derive(Debug, Default, Clone)]
enum DragTarget {
    TopThumb,

    #[default]
    BottomThumb,
}

#[derive(Debug, Default, Clone)]
pub struct TimelineScrollbarState {
    drag: Option<DragState>,

    viewport_start_ns: u64,
    viewport_end_ns: u64,

    zoom_start_ns: u64,
    zoom_end_ns: u64,
}

impl TimelineScrollbarState {
    fn thumb_top(&self, rect: Rectangle) -> Rectangle {
        self.thumb_viewport(self.zoom_start_ns, self.zoom_end_ns, rect)
    }

    fn thumb_bottom(&self, total_ns: u64, rect: Rectangle) -> Rectangle {
        self.thumb_viewport(0, total_ns, rect)
    }

    fn thumb_viewport(&self, start: u64, end: u64, rect: Rectangle) -> Rectangle {
        let x_start = ns_to_x(self.viewport_start_ns, start, end, rect);
        let x_end = ns_to_x(self.viewport_end_ns, start, end, rect);

        Rectangle {
            x: x_start,
            y: rect.y,
            width: (x_end - x_start).max(4.0),
            height: rect.height,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TimelineScrollbar<'a, Message, Theme>
where
    Theme: Catalog + 'a,
    Message: Clone,
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fixed(self.total_height()),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        Node::new(Size::new(limits.max().width, self.total_height()))
    }

    fn state(&self) -> State {
        State::new(TimelineScrollbarState {
            viewport_start_ns: 0,
            viewport_end_ns: self.window_width_ns,
            zoom_start_ns: 0,
            zoom_end_ns: self.zoom_width(),
            ..Default::default()
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TimelineScrollbarState>();
        let style = theme.style(&self.class);
        let bounds = layout.bounds();

        let mut top_rect = bounds.clone();
        top_rect.height = self.top_height;

        let mut bottom_rect = bounds.clone();
        bottom_rect.y += self.top_height;
        bottom_rect.height = self.bottom_height;

        // Draw the background color
        renderer.fill_quad(Quad {
            bounds,
            ..Default::default()
        }, style.background_color);

        // Draw the divider line between the top and bottom strips
        renderer.fill_quad(Quad {
            bounds: Rectangle {
                x: bounds.x,
                y: bounds.y + self.top_height - 1.0,
                width: bounds.width,
                height: 1.0,
            },
            ..Default::default()
        }, Color::from_rgba(1.0, 1.0, 1.0, 0.1));

        // In case the timeline has no duration yet, it is still loading --> TODO: Show loading indicator
        if self.total_ns == 0 {
            return;
        }

        // Draw top highlights (only those visible in zoom canvas)
        for h in self.highlights.iter().filter(|h| h.time_ns + h.duration.unwrap_or(0) >= state.zoom_start_ns && h.time_ns <= state.zoom_end_ns) {
            let expand = ns_to_x(h.duration.unwrap_or(0), 0, state.zoom_end_ns - state.zoom_start_ns, top_rect);
            renderer.fill_quad(Quad {
                bounds: Rectangle {
                    x: ns_to_x(h.time_ns, state.zoom_start_ns, state.zoom_end_ns, top_rect) - MARKER_POINTER_WIDTH_TOP / 2.0,
                    y: top_rect.y + 3.0,
                    width: MARKER_POINTER_WIDTH_TOP + expand,
                    height: top_rect.height - 6.0,
                },
                border: Border::default().rounded(2.0),
                ..Default::default()
            }, style.marker_style.marker_colors.get(&h.marker).cloned().unwrap_or(Color::TRANSPARENT));
        }

        // Draw bottom highlights (all recording highlights)
        for h in &self.highlights {
            let expand = ns_to_x(h.duration.unwrap_or(0), 0, self.total_ns, bottom_rect);
            renderer.fill_quad(Quad {
                bounds: Rectangle {
                    x: ns_to_x(h.time_ns, 0, self.total_ns, bottom_rect) - MARKER_POINTER_WIDTH_BOTTOM / 2.0,
                    y: bottom_rect.y + bottom_rect.height - 6.0,
                    width: MARKER_POINTER_WIDTH_BOTTOM + expand,
                    height: 6.0,
                },
                ..Default::default()
            }, style.marker_style.marker_colors.get(&h.marker).cloned().unwrap_or(Color::TRANSPARENT));
        }

        // Draw to top thumb
        renderer.fill_quad(Quad {
            bounds: state.thumb_top(top_rect),
            border: Border::default().rounded(4.0),
            ..Default::default()
        }, style.thumb_color);

        // Draw the bottom thumb
        renderer.fill_quad(Quad {
            bounds: state.thumb_bottom(self.total_ns, bottom_rect),
            border: Border::default().rounded(3.0),
            ..Default::default()
        }, style.thumb_color);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // In case the timeline has no duration yet, it is still loading --> TODO: Show loading indicator
        if self.total_ns == 0 {
            return;
        }

        let state = tree.state.downcast_mut::<TimelineScrollbarState>();
        let bounds = layout.bounds();

        let mut top_rect = bounds.clone();
        top_rect.height = self.top_height;

        let mut bottom_rect = bounds.clone();
        bottom_rect.y += self.top_height;
        bottom_rect.height = self.bottom_height;

        // Update the viewport position state if an update is pending
        if let Some((new_viewport_start, new_viewport_end)) = self.pending_update_position.take() {
            state.viewport_start_ns = new_viewport_start;
            state.viewport_end_ns = new_viewport_end;

            let duration_zoom = self.zoom_width();
            let duration_vp = state.viewport_end_ns.saturating_sub(state.viewport_start_ns);
            let progress = (new_viewport_start as f64 / (self.total_ns - duration_vp) as f64).clamp(0.0, 1.0);
            println!("{} // {}", duration_vp, duration_zoom);
            let offset_ns = ((duration_zoom - duration_vp) as f64 * progress) as u64;

            state.zoom_start_ns = new_viewport_start.saturating_sub(offset_ns);
            state.zoom_end_ns = (state.zoom_start_ns + duration_zoom).min(self.total_ns);
        }

        // Handle mouse interactions
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    let is_in_top = top_rect.contains(pos);
                    let is_in_bottom = bottom_rect.contains(pos);

                    if is_in_top || is_in_bottom {
                        let (thumb, target) = if is_in_top {(
                            state.thumb_top(top_rect),
                            DragTarget::TopThumb
                        )} else {(
                            state.thumb_bottom(self.total_ns, bottom_rect),
                            DragTarget::BottomThumb
                        )};

                        // Set new drag state
                        state.drag = Some(DragState {
                            target,
                            grab_offset_x: pos.x - thumb.x,
                            thumb_width: thumb.width
                        });

                        // Jump if click was outside thumb // TODO: When clicking on marker, it might be a little better if it would try to jump to the precise ns instead of the somewhat accurate
                        if !thumb.contains(pos) {
                            self.update_viewport(
                                state,
                                pos.x,
                                bounds,
                                shell,
                                true
                            );
                        }
                        return shell.capture_event();
                    };
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.drag.is_some() {
                    return self.update_viewport(
                        state,
                        position.x,
                        bounds,
                        shell,
                        false
                    );
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if state.drag.take().is_some() {
                    if let Some(ref on_position_set) = self.on_position_set {
                        shell.publish(on_position_set(state.viewport_start_ns, state.viewport_end_ns));
                    }
                    return shell.capture_event();
                }
            }
            _ => { }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> Interaction {
        let state = tree.state.downcast_ref::<TimelineScrollbarState>();

        let bounds = layout.bounds();
        if let Some(pos) = cursor.position() {
            let mut top_rect = bounds.clone();
            top_rect.height = self.top_height;

            let mut bottom_rect = bounds.clone();
            bottom_rect.y += self.top_height;
            bottom_rect.height = self.bottom_height;

            // Only show pointer when cursor is not on top of thumb
            let thumb_top = state.thumb_top(top_rect);
            let thumb_bottom = state.thumb_bottom(self.total_ns, bottom_rect);
            if thumb_top.contains(pos) || thumb_bottom.contains(pos) {
                return Interaction::default();
            }

            // Show pointer when cursor is on top of highlight
            if top_rect.contains(pos) {
                for h in self.highlights.iter().filter(|h| h.time_ns + h.duration.unwrap_or(0) >= state.zoom_start_ns && h.time_ns <= state.zoom_end_ns) {
                    let expand = ns_to_x(h.duration.unwrap_or(0), 0, state.zoom_end_ns - state.zoom_start_ns, top_rect);
                    let x = ns_to_x(h.time_ns, state.zoom_start_ns, state.zoom_end_ns, top_rect);
                    let distance = pos.x - x + MARKER_POINTER_WIDTH_TOP / 2.0;
                    if distance >= 0.0 && distance <= MARKER_POINTER_WIDTH_TOP + expand {
                        return Interaction::Pointer;
                    }
                }
            } else if bottom_rect.contains(pos) {
                for h in &self.highlights {
                    let expand = ns_to_x(h.duration.unwrap_or(0), 0, self.total_ns, bottom_rect);
                    let x = ns_to_x(h.time_ns, 0, self.total_ns, bottom_rect);
                    let distance = pos.x - x + MARKER_POINTER_WIDTH_BOTTOM / 2.0;
                    if distance >= 0.0 && distance <= MARKER_POINTER_WIDTH_BOTTOM + expand {
                        return Interaction::Pointer;
                    }
                }
            }
        }

        Interaction::default()
    }
}

impl<'a, Message, Theme> TimelineScrollbar<'a, Message, Theme>
where
    Message: Clone,
    Theme: Catalog + 'a
{
    /// Updates the viewport changed by dragging in the *top* or *bottom* strip based on the provided DragTarget.
    /// If a listener is available, the `on_viewport_change` message will be emitted
    fn update_viewport(
        &self,
        state: &mut TimelineScrollbarState,
        cursor_x: f32,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
        is_jump: bool
    ) {
        let Some(drag_state) = &state.drag else { return };

        let duration_zoom = self.zoom_width();
        let duration_vp = state.viewport_end_ns.saturating_sub(state.viewport_start_ns);
        let target = drag_state.target.clone();

        // Get offset and zoom
        let (zoom, (jump_start, jump_end)) = match target {
            DragTarget::TopThumb => (self.zoom_factor, (state.zoom_start_ns, state.zoom_end_ns)),
            DragTarget::BottomThumb => (1.0, (0, self.total_ns))
        };

        // Get new scroll progress
        let progress = if !is_jump {
            let new_start_x = cursor_x - drag_state.grab_offset_x;
            let max = bounds.width - (duration_vp as f32 / self.total_ns as f32) * bounds.width * zoom;
            let value = new_start_x - bounds.x;
            let progress = value as f64 / max as f64;

            progress.clamp(0.0, 1.0)
        } else {
            let new_start_x = cursor_x - (drag_state.thumb_width / 2.0);
            let new_start_ns = x_to_ns(new_start_x, jump_start, jump_end, bounds);
            let progress = new_start_ns as f64 / (self.total_ns - duration_vp) as f64;

            progress.clamp(0.0, 1.0)
        };

        // Get start and end ns
        let new_vp_start_ns = ((self.total_ns - duration_vp) as f64 * progress) as u64;
        let new_vp_end_ns = (new_vp_start_ns + duration_vp).min(self.total_ns);

        // Get the new start and end positions
        let offset_ns = ((duration_zoom - duration_vp) as f64 * progress) as u64;
        let new_zoom_start_ns = new_vp_start_ns.saturating_sub(offset_ns);
        let new_zoom_end_ns = (new_zoom_start_ns + duration_zoom).min(self.total_ns);

        // Update to new drag start offset position
        if is_jump && let Some(drag_state) = &mut state.drag {
            drag_state.grab_offset_x = match target {
                DragTarget::TopThumb => {
                    let prev_offset = (state.viewport_start_ns - state.zoom_start_ns) as f32;
                    let new_offset = (new_vp_start_ns - new_zoom_start_ns) as f32;
                    drag_state.grab_offset_x - (new_offset - prev_offset) / duration_zoom as f32 * bounds.width
                }
                DragTarget::BottomThumb => {
                    drag_state.thumb_width / 2.0
                }
            };
        };

        // Update state values
        state.viewport_start_ns = new_vp_start_ns;
        state.viewport_end_ns = new_vp_end_ns;
        state.zoom_start_ns = new_zoom_start_ns;
        state.zoom_end_ns = new_zoom_end_ns;

        // Emit viewport change if a listener is registered
        if let Some(ref on_viewport_change) = self.on_viewport_change {
            shell.publish(on_viewport_change(new_vp_start_ns, new_vp_end_ns));
        };

        // Emit zoom change if a listener is registered
        if let Some(ref on_zoom_change) = self.on_zoom_change {
            shell.publish(on_zoom_change(new_zoom_start_ns, new_zoom_end_ns));
        };

        // Capture the event
        shell.capture_event()
    }
}

impl<'a, Message, Theme, Renderer> From<TimelineScrollbar<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(widget: TimelineScrollbar<'a, Message, Theme>) -> Self {
        Element::new(widget)
    }
}

//#########################//
//         STYLING         //
//#########################//

/// The appearance of a [`Timeline`].
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// Color of the scrollbar background.
    pub background_color: Color,
    /// Color of the thumb in both strips.
    pub thumb_color: Color,
    /// Color of the thumb border.
    pub thumb_border_color: Color,
    /// Style of markers to display.
    pub marker_style: signal_markers::Style
}

/// The theme catalog of a [`Timeline`].
pub trait Catalog: Sized {
    /// The item class of this [`Catalog`].
    type Class<'a>;

    /// The default class produced by this [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, item: &Self::Class<'_>) -> Style;
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

pub fn default(theme: &Theme) -> Style {
    Style {
        background_color: Color::from_rgb(0.13, 0.13, 0.16),
        thumb_color: Color::from_rgba(0.45, 0.55, 0.75, 0.35),
        thumb_border_color: Color::from_rgba(0.55, 0.65, 0.85, 0.80),
        marker_style: signal_markers::default(theme)
    }
}
