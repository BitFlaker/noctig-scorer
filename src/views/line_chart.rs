use iced::{mouse, Color, Point, Rectangle, Renderer, Theme};
use iced::alignment::Vertical;
use iced::gradient::ColorStop;
use iced::widget::canvas::{Path, Cache, Frame, Geometry, Text};
use iced::widget::canvas::gradient::Linear;
use iced::widget::canvas;

use crate::storage::multi_reader::ChartSignal;
use crate::layout::scorer::SIGNAL_PADDING_VERTICAL;
use crate::formatting::font::REGULAR_BOLD;

pub struct Liner {
    signal_index: usize,
    label: String,
    range: [String; 2],
    draw_ranges: bool,
    data_min: (f32, f32),
    data_max: (f32, f32),
    count_before: u8, 
    count_after: u8,
    points: Vec<(f32, f32)>,
    cache: canvas::Cache,
}

impl Liner {
    pub fn from_chart_signal(chart_signal: ChartSignal, base_index: usize, draw_ranges: bool, count_before: u8, count_after: u8) -> Self {
        Self {
            signal_index: base_index + chart_signal.index,
            label: chart_signal.label,
            range: chart_signal.range,
            data_min: (0.0, chart_signal.physical_min as f32),
            data_max: (chart_signal.points.len() as f32, chart_signal.physical_max as f32),
            count_before, 
            count_after,
            points: chart_signal.points,
            draw_ranges,
            cache: Cache::new(),
        }
    }
}

impl<Message> canvas::Program<Message> for Liner {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &Renderer, theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        const RANGE_OFFSET_Y: f32 = 10.0;

        // Use cache so geometry is only rebuilt when cache is invalidated.
        let geometry = self.cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
            let segment_count = self.count_before as u16 + self.count_after as u16 + 1;
            let segment_percent = 1.0 / segment_count as f32;
            frame.fill_rectangle(Point::ORIGIN, frame.size(), canvas::Gradient::Linear(Linear { 
                start: Point::ORIGIN,
                end: Point::new(bounds.width, 0.0), 
                stops: [
                    Some(ColorStop { 
                        offset: 0.0, 
                        color: Color::TRANSPARENT
                    }), 
                    Some(ColorStop { 
                        offset: segment_percent * self.count_before as f32, 
                        color: Color::TRANSPARENT
                    }), 
                    Some(ColorStop { 
                        offset: segment_percent * self.count_before as f32, 
                        color: Color { 
                            r: 0.21,
                            g: 0.21,
                            b: 0.23,
                            a: 1.0 
                        } 
                    }), 
                    Some(ColorStop { 
                        offset: segment_percent * (self.count_before as f32 + 1.0), 
                        color: Color { 
                            r: 0.21,
                            g: 0.21,
                            b: 0.23,
                            a: 1.0 
                        } 
                    }), 
                    Some(ColorStop { 
                        offset: segment_percent * (self.count_before as f32 + 1.0), 
                        color: Color::TRANSPARENT
                    }), 
                    Some(ColorStop { 
                        offset: 1.0, 
                        color: Color::TRANSPARENT
                    }), 
                    None, None] 
                }
            ));
            
            frame.stroke(&Path::line(
                Point { 
                    x: frame.width() * segment_percent * self.count_before as f32, 
                    y: 0.0 
                }, 
                Point { 
                    x: frame.width() * segment_percent * self.count_before as f32, 
                    y: frame.height() 
                }), 
                canvas::Stroke::default().with_color(Color { 
                    r: 0.31,
                    g: 0.31,
                    b: 0.35,
                    a: 1.0 
                })
            );
            frame.stroke(&Path::line(
                Point { 
                    x: frame.width() * segment_percent * (self.count_before as f32 + 1.0), 
                    y: 0.0 
                }, 
                Point { 
                    x: frame.width() * segment_percent * (self.count_before as f32 + 1.0), 
                    y: frame.height() 
                }), 
                canvas::Stroke::default().with_color(Color { 
                    r: 0.31,
                    g: 0.31,
                    b: 0.35,
                    a: 1.0 
                })
            );

            // Draw background highlight color for odd indices to differentiate
            if self.signal_index % 2 != 0 {
                let mut highlight = theme.extended_palette().background.strongest.color;
                highlight.a = 0.16;
                frame.fill_rectangle(Point::ORIGIN, frame.size(), highlight);
            }

            // Draw min / max values if desired 
            if self.draw_ranges {
                // Draw max value
                let mut max_text = Text::from(self.range[1].clone());
                max_text.size = 14.0.into();
                max_text.color = Color::from_rgb8(100, 100, 100);
                max_text.align_y = Vertical::Top;
                max_text.align_x = iced::widget::text::Alignment::Right;
                max_text.position = Point::new(frame.width() - 10.0,  RANGE_OFFSET_Y);
                frame.fill_text(max_text);
    
                // Draw min value
                let mut min_text = Text::from(self.range[0].clone());
                min_text.size = 14.0.into();
                min_text.color = Color::from_rgb8(100, 100, 100);
                min_text.align_y = Vertical::Bottom;
                min_text.align_x = iced::widget::text::Alignment::Right;
                min_text.position = Point::new(frame.width() - 10.0, frame.height() - RANGE_OFFSET_Y);
                frame.fill_text(min_text);
            }

            // Draw line label
            let mut label_text = Text::from(self.label.clone());
            label_text.size = 16.0.into();
            label_text.font = *REGULAR_BOLD;
            label_text.color = Color::from_rgb8(100, 100, 100);
            label_text.align_y = Vertical::Bottom;
            // label_text.position = Point::new(10.0,  frame.height() / 2.0);
            label_text.position = Point::new(8.0,  frame.height() - 8.0);
            frame.fill_text(label_text.clone());

            // Draw actual line
            let (min_x, min_y) = self.data_min;
            let (max_x, max_y) = self.data_max;
            let width = frame.width().max(1.0);
            let height = (frame.height() - 2.0 * SIGNAL_PADDING_VERTICAL).max(1.0);
            let data_w = (max_x - min_x).max(1e-12);
            let data_h = (max_y - min_y).max(1e-12);

            let mut builder = canvas::path::Builder::new();
            let mut first = true;
            let mut last_pixel: Option<(i32, i32)> = None;

            for &(x, y) in &self.points {
                if y.is_nan() { 
                    continue; 
                }

                // Project to canvas coordinate (sx, sy)
                let sx = ((x - min_x) / data_w) * width;
                // flip Y so higher data y appears at top; adjust to your preference
                let sy = height - ((y - min_y) / data_h) * height + SIGNAL_PADDING_VERTICAL;

                // Pixel coordinates (rounded)
                let px = (sx.round() as i32, sy.round() as i32);

                // If pixel same as last, skip due to no visible change
                if Some(px) == last_pixel {
                    continue;
                }
                last_pixel = Some(px);

                if first {
                    builder.move_to(Point::new(sx as f32, sy as f32));
                    first = false;
                } else {
                    builder.line_to(Point::new(sx as f32, sy as f32));
                }
            }

            // Build and stroke the path if there are segments
            if !first {
                let path = builder.build();
                frame.stroke(&path, canvas::Stroke {
                    width: 1.5,
                    style: canvas::Style::Gradient(canvas::Gradient::Linear(
                        Linear { 
                            start: Point::ORIGIN,
                            end: Point::new(bounds.width, 0.0), 
                            stops: [
                                Some(ColorStop { 
                                    offset: 0.0, 
                                    color: Color { 
                                        r: 0.36, 
                                        g: 0.36,
                                        b: 0.36,
                                        a: 1.0
                                    } 
                                }), 
                                Some(ColorStop { 
                                    offset: segment_percent * self.count_before as f32, 
                                    color: Color { 
                                        r: 0.36, 
                                        g: 0.36,
                                        b: 0.36,
                                        a: 1.0
                                    } 
                                }), 
                                Some(ColorStop { 
                                    offset: segment_percent * self.count_before as f32, 
                                    color: Color { 
                                        r: 0.26,
                                        g: 0.36,
                                        b: 0.76,
                                        a: 1.0 
                                    } 
                                }), 
                                Some(ColorStop { 
                                    offset: segment_percent * (self.count_before as f32 + 1.0), 
                                    color: Color { 
                                        r: 0.26,
                                        g: 0.46,
                                        b: 0.86,
                                        a: 1.0 
                                    } 
                                }), 
                                Some(ColorStop { 
                                    offset: segment_percent * (self.count_before as f32 + 1.0), 
                                    color: Color { 
                                        r: 0.36, 
                                        g: 0.36,
                                        b: 0.36,
                                        a: 1.0
                                    } 
                                }), 
                                Some(ColorStop { 
                                    offset: 1.0, 
                                    color: Color { 
                                        r: 0.36, 
                                        g: 0.36,
                                        b: 0.36,
                                        a: 1.0
                                    } 
                                }), 
                                None, None] 
                            }
                        )
                    ),
                    ..canvas::Stroke::default()
                });
            }

            // // Line shading for label
            // let mut size = frame.size();
            // size.width = 80.0;
            // frame.fill_rectangle(Point::ORIGIN, size, Gradient::Linear(
            //     Linear::new(Point::ORIGIN, Point { x: size.width, y: 0.0 })
            //         .add_stop(0.0, Color::from_rgba8(32, 32, 32, 0.88))
            //         .add_stop(0.8, Color::from_rgba8(32, 32, 32, 0.88))
            //         .add_stop(1.0, Color::from_rgba8(32, 32, 32, 0.0))
            //     )
            // );
        });

        vec![geometry]
    }
}