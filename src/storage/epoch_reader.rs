use edf_rs::record::{RelativeRecordData, SpanningRecord};
use edf_rs::{file::EDFFile, headers::signal_header::SignalHeader};
use log::warn;
use std::{error::Error, iter::repeat_n, path::Path};

pub struct ChartSignal {
    pub index: usize,
    pub label: String,
    pub range: [String; 2],
    pub points: Vec<(f32, f32)>,
    pub physical_min: f64,
    pub physical_max: f64,
}

pub struct EpochReader {
    file: EDFFile,
    start_align_offset: u64,    // ms
    offset: u64,
    position: i128,  // ms
    last_epoch_count: usize,
    record: SpanningRecord
}

impl EpochReader {
    pub const EPOCH_DURATION: u32 = 30;

    pub fn new<P: AsRef<Path>>(path: P) -> std::result::Result<Self, Box<dyn Error>> {
        let file = EDFFile::open(path)?;

        Ok(Self {
            record: SpanningRecord::new(&file.header),
            file,
            start_align_offset: 0,
            offset: 0,
            position: 0,
            last_epoch_count: 0,
        })
    }

    pub fn set_start_align_offset(&mut self, offset: u64) {
        self.position += self.start_align_offset as i128;
        self.start_align_offset = offset;
        self.position -= self.start_align_offset as i128;
    }

    pub fn get_start_align_offset(&self) -> u64 {
        self.start_align_offset
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.position += self.offset as i128;
        self.offset = offset;
        self.position -= self.offset as i128;
    }

    pub fn get_epoch_count(&self) -> u64 {
        let Some(signal) = self.file.header.get_signals().get(0) else {
            return 0;
        };
        let samples_per_second = signal.samples_count as f64 / self.file.header.get_record_duration();
        let samples_per_epoch = samples_per_second * Self::EPOCH_DURATION as f64;
        let total_sample_count = signal.samples_count as u128 * self.file.header.get_record_count().unwrap_or(0) as u128;
        (total_sample_count + self.offset as u128).div_ceil(samples_per_epoch as u128) as u64
    }

    pub fn get_start_align_epoch_count(&self) -> u64 {
        self.start_align_offset as u64
    }

    pub fn get_window_start_epoch(&self) -> u64 {
        (self.tell() / (Self::EPOCH_DURATION as i128 * 1_000)) as u64 - self.last_epoch_count as u64
    }

    pub fn get_window_end_epoch(&self) -> u64 {
        (self.tell() / (Self::EPOCH_DURATION as i128 * 1_000)) as u64 - self.get_start_align_epoch_count()
    }

    pub fn get_epoch_sample_count(&self) -> u64 {
        let Some(signal) = self.file.header.get_signals().get(0) else {
            return 0;
        };
        let samples_per_second = signal.samples_count as f64 / self.file.header.get_record_duration();
        (samples_per_second * Self::EPOCH_DURATION as f64) as u64
    }

    pub fn start_timestamp(&self) -> u64 {
        self.file.header.start_date().and_time(self.file.header.get_start_time()).and_utc().timestamp() as u64
    }

    pub fn current_timeframe(&self) -> (u64, u64) {
        let Some(signal) = self.file.header.get_signals().get(0) else {
            return (0, 0);
        };

        let samples_per_second = signal.samples_count as f64 / self.file.header.get_record_duration();
        let epoch_samples = (samples_per_second * Self::EPOCH_DURATION as f64) as u64;
        let window_samples = epoch_samples; // * (epochs before + after + 1) ??
        let window_start = self.get_window_start_epoch() * epoch_samples;
        let window_end = window_start + window_samples;
        let window_start_s = window_start / samples_per_second as u64;
        let window_end_s = window_end / samples_per_second as u64;

        (window_start_s, window_end_s)
    }

    pub fn get_chart_signals(&self) -> Vec<ChartSignal> {
        self.get_signals()
            .iter()
            .filter(|s| !s.is_annotation())
            .enumerate()
            .map(|(i, signal)| ChartSignal {
                index: i,
                label: signal.label.clone(),
                range: [
                    format!("{} {}", signal.physical_minimum, signal.physical_dimension),
                    format!("{} {}", signal.physical_maximum, signal.physical_dimension)
                ],
                points: self.record.raw_signal_samples.get(i)
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|record_data| to_physical(signal, record_data))  // TODO: In case of discontinuous EDFs, fill gaps or similar
                    .flatten()
                    .enumerate()
                    .map(|(i, value)| (i as f32, value.clone() as f32))
                    .collect::<Vec<_>>(),
                physical_min: signal.physical_minimum,
                physical_max: signal.physical_maximum,
            })
            .collect::<Vec<_>>()
    }

    pub fn get_signals(&self) -> &Vec<SignalHeader> {
        &self.file.header.get_signals()
    }

    pub fn tell(&self) -> i128 {
        self.position + self.offset as i128 + self.start_align_offset as i128
    }

    pub fn file(&self) -> &EDFFile {
        &self.file
    }

    pub fn read_epochs(&mut self, count: usize) -> std::result::Result<(), Box<dyn Error>> {
        let read_millis = count as u128 * Self::EPOCH_DURATION as u128 * 1_000;

        let is_padded_start = self.position < 0;
        let actual_millis = if is_padded_start { read_millis.saturating_sub(self.position.abs() as u128) } else { read_millis };
        self.last_epoch_count = count;
        // TODO: Is padded both sides

        let mut spanning_record = self.file.read_millis(actual_millis)?;

        // Add padding values to the beginning and/or the and in case the returned samples are less than count
        let mut signal_idx = 0;
        for signal in self.file.header.get_signals() {
            if signal.is_annotation() {
                continue;
            }

            // Get desired amount of padding samples
            let target = spanning_record.raw_signal_samples.get_mut(signal_idx).expect("Spanning record layout must match file layout");
            let target_sample_count = (signal.samples_count as f64 * read_millis as f64 / (self.file.header.get_record_duration() * 1000.0)) as usize;
            let actual_sample_count = target.iter().fold(0, |acc, x| acc + x.raw_signal_samples.len());
            let sample_pad_count = target_sample_count - actual_sample_count;
            if sample_pad_count != 0 {
                let padding_samples = repeat_n(0, sample_pad_count).collect::<Vec<_>>();
                let pad_record = RelativeRecordData {
                    offset: f64::NAN,
                    raw_signal_samples: padding_samples
                };

                // Insert padding samples in correct position
                if is_padded_start {
                    target.insert(0, pad_record);
                }
                else {
                    target.push(pad_record);
                }
            }

            signal_idx += 1;
        }

        self.record = spanning_record;
        self.position += read_millis as i128;

        Ok(())
    }

    pub fn seek(&mut self, millis: u64) -> std::result::Result<(), Box<dyn Error>> {
        let position = millis as i128 - self.offset as i128 - self.start_align_offset as i128;

        if position >= 0 {
            let record_millis = (self.file.header.get_record_duration() * 1_000.0) as u64;
            let record = (position as u64 / record_millis) as usize;
            let record_skip_time = millis % record_millis;
            self.file.seek_to_record(record)?;
            _ = self.file.read_millis(record_skip_time as u128)?;
        }
        else {
            self.file.seek_to_record(0)?;
        }

        self.position = position;

        // TODO: Maybe clear current record buffer

        Ok(())
    }

    pub fn signal_count(&self) -> usize {
        self.file.header.get_signals().len()
    }
}

fn to_physical(signal: &SignalHeader, record_data: &RelativeRecordData) -> Vec<f64> {
    if record_data.offset.is_nan() {
        return vec![f64::NAN; record_data.raw_signal_samples.len()];
    }

    record_data.get_physical_samples(signal)
}
