use std::time::Duration;
use iced::{mouse, Point, Rectangle, Renderer, Theme};
use iced::widget::canvas::{Path, Cache, Frame, Geometry};
use iced::widget::canvas;

pub struct Timeline {
    project_duration: Option<Duration>,
    marker_positions: Vec<f32>,
    annotation_positions: Vec<(f32, f32)>,
    window_size: f32,
    position: f32,
    is_sliding: bool,
    cache: canvas::Cache,
}

impl Timeline {
    pub fn new(window_size: f32, markers: Vec<f32>, annotations: Vec<(f32, f32)>) -> Self {
        Self {
            window_size,
            project_duration: None,
            marker_positions: markers,
            annotation_positions: annotations,
            position: 0.0,
            is_sliding: false,
            cache: Cache::new()
        }
    }

    /// Sets the starting position of the window (in seconds) currently displayed in the viewport.
    pub fn set_position(&mut self, position: f32) {
        self.position = position;
    }

    /// Sets the size of the window (in seconds) currently displayed in the current viewport.
    /// This represents the currently visible portion of the entire timeline.
    pub fn set_window_size(&mut self, window_size: f32) {
        self.window_size = window_size;
    }

    /// Sets the duration of the entire open project. If the value is `None` the timeline will
    /// be put into a loading mode and show a loading animation until the duration is set to a value again.
    /// This is the duration the markers and annotations will be mapped on to
    pub fn set_project_duration(&mut self, project_duration: Option<Duration>) {
        self.project_duration = project_duration;
    }

    /// Sets the timestamps (in seconds) where a marker should be displayed
    pub fn set_markers(&mut self, timestamps: Vec<f32>) {
        self.marker_positions = timestamps;
    }

    /// Sets the timestamps (in seconds) as well as its duration (in seconds) where an annotation
    /// marker should be displayed
    pub fn set_annotations(&mut self, timestamp_durations: Vec<(f32, f32)>) {
        self.annotation_positions = timestamp_durations;
    }
}

impl<Message> canvas::Program<Message> for Timeline {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &Renderer, theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
            let palette = theme.extended_palette();
            let background = palette.background.weakest.color;
            let border = palette.background.strong.color;

            // View background and border
            frame.fill_rectangle(Point::ORIGIN, frame.size(), background);
            frame.stroke(&Path::line(
                    Point::ORIGIN,
                    Point::new(frame.width(), 0.0)
                ),
                canvas::Stroke::default().with_color(border)
            );


        });

        vec![geometry]
    }
}
