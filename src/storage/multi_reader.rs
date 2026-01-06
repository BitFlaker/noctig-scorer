use edfplus::{EdfError, EdfHeader, EdfReader, SignalParam};
use std::{error::Error, iter::repeat_n, path::Path};

pub struct ChartSignal {
    pub index: usize, 
    pub label: String,
    pub range: [String; 2],
    pub points: Vec<(f32, f32)>,
    pub physical_min: f64,
    pub physical_max: f64,
}

pub struct Signal {
    pub label: String,
    pub physical_min: f64,
    pub physical_max: f64,
    pub physical_unit: String,
    pub samples: Vec<f64>
}

impl Signal {
    pub fn empty() -> Self {
        Self {
            label: String::new(),
            physical_min: 0.0,
            physical_max: 0.0,
            physical_unit: String::new(),
            samples: Vec::new()
        }
    }
}

pub struct MultiEdfReader {
    readers: Vec<EdfReader>,
    header: EdfHeader,
    signals: Vec<Signal>,
    start_align_offset: u64,
    offset: u64,
    position: i64,
    last_epoch_count: u64
}

impl MultiEdfReader {
    pub const EPOCH_DURATION: u32 = 30;

    pub fn from_paths<P>(paths: &[P]) -> std::result::Result<Self, Box<dyn Error>> where P: AsRef<Path> {
        let mut readers = Vec::with_capacity(paths.len());
        let mut signals = Vec::with_capacity(paths.len());
        let mut header: Option<EdfHeader> = None;

        if paths.len() == 0 {
            return Err("No Paths provided!".into());
        }

        // Load all signals from all paths
        for path in paths {
            // TODO: Ensure equivalent config / headers
            let reader = EdfReader::open(path)?;

            match &mut header {
                Some(header) => {
                    header.signals.extend(reader.header().signals.clone());
                },
                _ => {
                    header = Some(clone_header(reader.header()));
                }
            }

            readers.push(reader);
            signals.push(Signal::empty());
        }

        Ok(Self {
            readers,
            header: header.unwrap(),
            signals,
            start_align_offset: 0,
            offset: 0,
            position: 0,
            last_epoch_count: 0
        })
    }

    pub fn set_start_align_offset(&mut self, offset: u64) {
        self.position += self.start_align_offset as i64;
        self.start_align_offset = offset;
        self.position -= self.start_align_offset as i64;
    }

    pub fn get_start_align_offset(&self) -> u64 {
        self.start_align_offset
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.position += self.offset as i64;
        self.offset = offset;
        self.position -= self.offset as i64;
    }

    pub fn get_epoch_count(&self) -> u64 {
        let samples_per_second = self.header.signals[0].samples_per_record as i64 / (self.header.datarecord_duration / 10_000_000);
        let samples_per_epoch = samples_per_second as u64 * Self::EPOCH_DURATION as u64;
        let total_sample_count = (self.header.signals[0].samples_per_record as i64 * self.header.datarecords_in_file).abs();
        (total_sample_count as u64 + self.offset).div_ceil(samples_per_epoch)
    }

    pub fn get_start_align_epoch_count(&self) -> u64 {
        self.start_align_offset as u64 / self.get_epoch_sample_count()
    }

    pub fn get_current_epoch(&self) -> u64 {
        (self.tell().unwrap() / self.get_epoch_sample_count() as i64) as u64 - self.last_epoch_count
    }

    pub fn tell_epoch(&self) -> u64 {
        (self.tell().unwrap() / self.get_epoch_sample_count() as i64) as u64 - self.get_start_align_epoch_count()
    }

    pub fn get_epoch_sample_count(&self) -> u64 {
        let samples_per_second = self.header.signals[0].samples_per_record as i64 / (self.header.datarecord_duration / 10_000_000);
        samples_per_second as u64 * Self::EPOCH_DURATION as u64
    }

    pub fn start_timestamp(&self) -> u64 {
        self.header.start_date.and_time(self.header.start_time).and_utc().timestamp() as u64
    }

    pub fn current_timeframe(&self) -> (u64, u64) {
        let samples_per_second = (self.header.signals[0].samples_per_record as i64 / (self.header.datarecord_duration / 10_000_000)).abs() as u64;
        let window_samples = samples_per_second * Self::EPOCH_DURATION as u64;
        let window_start = self.get_current_epoch() * self.get_epoch_sample_count();
        let window_end = window_start + window_samples;
        let window_start_s = (window_start / samples_per_second) as u64;
        let window_end_s = (window_end / samples_per_second) as u64;

        (window_start_s, window_end_s)
    }

    pub fn get_chart_signals(&self) -> Vec<ChartSignal> {
        self.get_signals().iter().enumerate().map(|(i, signal)| ChartSignal {
            index: i, 
            label: signal.label.clone(), 
            range: [format!("{} {}", signal.physical_min, signal.physical_unit), format!("{} {}", signal.physical_max, signal.physical_unit)], 
            points: signal.samples.iter().enumerate().map(|(i, value)| (i as f32, value.clone() as f32)).collect::<Vec<_>>(),
            physical_min: self.header().signals.first().unwrap().physical_min,
            physical_max: self.header().signals.first().unwrap().physical_max,
        }).collect::<Vec<_>>()
    }
    
    pub fn get_signals(&self) -> &Vec<Signal> {
        &self.signals
    }

    pub fn header(&self) -> &EdfHeader {
        &self.header
    }

    pub fn tell(&self) -> Result<i64, EdfError> { 
        Ok(self.position + self.offset as i64 + self.start_align_offset as i64)
        // self.readers.first().unwrap().tell(0)
    }

    pub fn read_physical_samples(&mut self, count: usize) -> std::result::Result<(), Box<dyn Error>>{
        let is_padded_start = self.position < 0;
        let actual_count = if is_padded_start { count.saturating_sub(self.position.abs() as usize) } else { count };
        self.last_epoch_count = count as u64 / self.get_epoch_sample_count();
        // TODO: Is padded both sides

        let mut signals = Vec::with_capacity(self.signals.len());
        for i in 0..self.signal_count() {
            let Some((_start_index, reader)) = self.get_signal_reader_mut(i) else {
                return Err("".into());
            };
            let sig_count = reader.header().signals.len();
            for j in 0..sig_count {
                let signal = &reader.header().signals[j];
                let label = signal.label.clone();
                let physical_max = signal.physical_max;
                let physical_min = signal.physical_min;
                let physical_unit = signal.physical_dimension.clone();
                let mut samples = reader.read_physical_samples(j, actual_count)?;

                // Add padding values to the beginning and/or the and in case the returned samples are less than count
                if samples.len() < count {
                    if is_padded_start {
                        let mut new_samples = repeat_n(f64::NAN, count - samples.len()).collect::<Vec<_>>();
                        new_samples.extend(samples);
                        samples = new_samples;
                    }
                    else {
                        samples.extend(repeat_n(f64::NAN, count - samples.len()));
                    }
                }

                signals.push(Signal { 
                    label, 
                    physical_max,
                    physical_min,
                    physical_unit,
                    samples 
                });
            }
        }
        self.signals = signals;
        self.position += count as i64;

        Ok(())
    }

    pub fn seek(&mut self, position: i64) -> std::result::Result<(), Box<dyn Error>> {
        let position = position.max(0) - self.offset as i64 - self.start_align_offset as i64;
        for i in 0..self.signal_count() {
            let Some((_start_index, reader)) = self.get_signal_reader_mut(i) else {
                return Err("".into());
            };
            let sig_count = reader.header().signals.len();
            for j in 0..sig_count {
                reader.seek(j, position)?;
            }
        }
        self.position = position;

        Ok(())
    }

    pub fn get_signal_param(&self, index: usize) -> Option<&SignalParam> {
        let Some((start_index, reader)) = self.get_signal_reader(index) else {
            return None;
        };
        Some(&reader.header().signals[index - start_index])
    }

    pub fn signal_count(&self) -> usize {
        self.readers.iter().map(|r| r.header().signals.len()).sum()
    }

    fn get_signal_reader(&self, index: usize) -> Option<(usize, &EdfReader)> {
        let mut i = 0;
        for reader in &self.readers {
            let sig_count = reader.header().signals.len();
            if i <= index && sig_count >= index {
                return Some((i, reader));
            }
            i += sig_count;
        }
        return None;
    }

    fn get_signal_reader_mut(&mut self, index: usize) -> Option<(usize, &mut EdfReader)> {
        let mut i = 0;
        for reader in &mut self.readers {
            let sig_count = reader.header().signals.len();
            if i <= index && i + sig_count > index {
                return Some((i, reader));
            }
            i += sig_count;
        }
        return None;
    }
}

// Required because EdfHeader is not Clone
pub fn clone_header(header: &EdfHeader) -> EdfHeader {
    EdfHeader { 
        signals: header.signals.clone(), 
        file_duration: header.file_duration, 
        start_date: header.start_date, 
        start_time: header.start_time, 
        starttime_subsecond: header.starttime_subsecond, 
        datarecords_in_file: header.datarecords_in_file, 
        datarecord_duration: header.datarecord_duration, 
        annotations_in_file: header.annotations_in_file, 
        patient_code: header.patient_code.clone(), 
        sex: header.sex.clone(), 
        birthdate: header.birthdate.clone(), 
        patient_name: header.patient_name.clone(), 
        patient_additional: header.patient_additional.clone(),
        admin_code: header.admin_code.clone(), 
        technician: header.technician.clone(), 
        equipment: header.equipment.clone(), 
        recording_additional: header.recording_additional.clone() 
    }
}