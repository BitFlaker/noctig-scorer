use iced::widget::shader::Program;
use iced::{Rectangle, mouse};

use crate::views::spectrogram::widget::shader_handler::SpectrogramViewPrimitive;
use crate::external::yasa::plot_spectrogram;
use crate::external::scipy::Spectrogram;
use crate::Message;

pub mod color_maps;
pub mod shader_handler;

#[derive(Debug, Default, Clone)]
pub struct SpectrogramView {
    freqs: Vec<f32>,
    time: Vec<f32>,
    result: Vec<f32>,
    vmin: f64,
    vmax: f64,
    color_map: String
}

impl SpectrogramView {
    pub fn new(spectrogram: Spectrogram, color_map: String) -> Self {
        let (spectrogram, (vmin, vmax)) = plot_spectrogram(spectrogram);
        let result = spectrogram.result.as_standard_layout().iter().map(|v| *v as f32).collect();
        let freqs = spectrogram.freqs.iter().map(|x| *x as f32).collect();
        let time = spectrogram.time.iter().map(|x| *x as f32).collect();

        Self {
            vmin,
            vmax,
            freqs,
            time,
            result,
            color_map
        }
    }

    pub fn from_spectrogram(spectrogram: Spectrogram) -> Self {
        Self::new(spectrogram, "lajolla".to_string())
    }

    pub fn set_colormap(&mut self, color_map: String) {
        self.color_map = color_map;
    }
}

impl Program<Message> for SpectrogramView {
    type State = ();
    type Primitive = SpectrogramViewPrimitive;

    fn draw(&self, _state: &Self::State, _cursor: mouse::Cursor, _bounds: Rectangle) -> Self::Primitive {
        SpectrogramViewPrimitive::new(self.clone())
    }
}
