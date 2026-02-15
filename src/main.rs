#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]  // hide console window on Windows in release

use edf_rs::file::EDFFile;
use iced::futures::channel::mpsc::Sender;
use iced::futures::{SinkExt, Stream};
use iced::window::{Id, Position, Settings};
use iced::{Element, Point, Size, Subscription, Task, Theme, Vector, window};
use iced::keyboard::{key::Named, Key};
use iced::event::Status;
use iced::event;
use iced::widget::{self, space};
use ndarray::Array1;
use rfd::AsyncFileDialog;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use env_logger::Builder;
use log::{LevelFilter, warn};
use iced::stream::channel;

use crate::database::types::RecentProject;
use crate::database::{get_last_browse_source_path, get_last_project_create_path, get_last_project_path, get_recently_opened, set_last_browse_source_path, set_last_project_create_path, set_last_project_path, update_recently_opened};
use crate::external::lspopt::spectrogram_lspopt;
use crate::external::scipy::Spectrogram;
use crate::layout::create_project::create_viewer;
use crate::layout::license::load_licenses;
use crate::layout::{license, scorer, start};
use crate::storage::epoch_reader::EpochReader;
use crate::formatting::theme::{CLEAR_DARK_TEXT_SECONDARY, border_background_base, text_foreground_base};
use crate::storage::project_initializer;
use crate::views::collapsible::Collapsible;
use crate::views::spectrogram::widget::SpectrogramView;

mod layout;
mod formatting;
mod storage;
mod views;
mod macros;
mod external;
mod database;

pub const ICON: &[u8] = include_bytes!("../resources/icon.svg");
pub const ICON_SECONDARY: LazyLock<Vec<u8>> = LazyLock::new(|| include_str!("../resources/icon.svg").replace("fill:#ffffff", &format!("fill:{}", CLEAR_DARK_TEXT_SECONDARY.to_string())).replace("stroke:#ffffff", &format!("stroke:{}", CLEAR_DARK_TEXT_SECONDARY.to_string())).into_bytes());

// TODO: Setting for preventing to cache the last file picker dialog location

fn main() -> iced::Result {
    // Configure logger
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter(Some(env!("CARGO_PKG_NAME")), LevelFilter::Info)
        .init();

    // Launch UI
    iced::daemon(NoctiG::boot, NoctiG::update, NoctiG::view)
        .title("NoctiG Scorer")
        .theme(NoctiG::theme)
        .subscription(NoctiG::subscription)
        .settings(settings())
        .run()
}

fn settings() -> iced::Settings {
    iced::Settings {
        id: None,
        antialiasing: true,
        .. Default::default()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Scorings {
    pub epoch_duration: u64,

    // Mapping beteen epoch segment index and its determined stage
    pub values: BTreeMap<u64, Stage>
}

#[derive(Serialize, Deserialize, Default)]
pub struct Markers {
    pub global: HashMap<Marker, Vec<u64>>,
    pub local: HashMap<u32, HashMap<Marker, Vec<u64>>>
}

#[derive(Serialize, Deserialize, Default)]
pub struct Annotations {
    pub global: HashMap<Marker, Vec<AnnotationValue>>,
    pub local: HashMap<u32, HashMap<Marker, Vec<AnnotationValue>>>
}

#[derive(Serialize, Deserialize, Default)]
pub struct AnnotationValue {
    pub timestamp: u64,
    pub value: String
}

#[derive(Serialize, Deserialize, Default)]
pub struct SessionState {
    pub position: u64

    // TODO: Save the toggle states and current settings (e.g. timeframe format, show legend, etc.)
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub project_type: ProjectType,
    pub epoch_duration: u64,
    pub epochs_before_current: u8,
    pub epochs_after_current: u8,
    pub filter_signal: bool,
    pub clip_signal: bool,
    pub auto_align_signals: bool,
    pub signals: Vec<SignalSource>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Project {
    pub fn from_config(config: &ProjectConfiguration) -> Self {
        Self::from_config_and_signals(config, &config.data)
    }

    pub fn from_config_and_signals(config: &ProjectConfiguration, signals: &Vec<ProjectSignals>) -> Self {
        Self {
            name: config.name.clone(),
            project_type: ProjectType::SleepScoring,
            tags: config.tags.iter().cloned().collect(),
            epoch_duration: 30,
            epochs_before_current: 1,
            epochs_after_current: 1,
            signals: signals.iter().map(SignalSource::from_config).collect(),
            filter_signal: config.filter_signal,
            auto_align_signals: config.auto_align_signals,
            clip_signal: config.clip_signal,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SignalSource {
    pub path: String,
    pub read_only: bool,
    pub offset: u64,
    #[serde(default)]
    pub merge_groups: Vec<SignalMergeGroup>
}

impl SignalSource {
    pub fn from_config(config: &ProjectSignals) -> Self {
        let path = if config.is_reference {
            config.path.clone()
        } else {
            Path::new("sources").join(&config.name).to_string_lossy().to_string()
        };

        Self {
            path,
            read_only: config.is_reference,
            offset: 0,
            merge_groups: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub enum WindowType {
    MainWindow,
    Licenses,
    About
}

impl WindowType {
    pub fn settings(&self) -> window::Settings {
        match self {
            WindowType::MainWindow => Settings {
                position: Position::Centered,
                size: Size::new(1400.0, 800.0),
                ..Default::default()
            },
            WindowType::Licenses => Settings {
                size: Size::new(900.0, 600.0),
                ..Default::default()
            },
            WindowType::About => Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SignalMergeGroup {
    pub signal_id: u16,
    pub group_id: u16
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    SleepScoring,
    EKG
}

pub struct ProjectConfiguration {
    pub name: String,
    pub path: String,
    pub new_tag: String,
    pub tags: Vec<String>,
    pub data: Vec<ProjectSignals>,
    pub filter_signal: bool,
    pub clip_signal: bool,
    pub auto_align_signals: bool,
}

pub struct ProjectSignals {
    pub timestamp: u64,
    pub duration: f64,
    pub signal_count: usize,
    pub path: String,
    pub name: String,
    pub is_reference: bool
}

pub struct CurrentProject {
    path: String,
    project_name: String,
    readers: Vec<EpochReader>,
    project: Project,
    markers: Markers,
    annotations: Annotations,
    scorings: Option<Scorings>,
    spectrogram: Option<SpectrogramView>,
    loading_progress_spectrogram: Option<f32>
}

pub struct LicenseData {
    pub url: String,
    pub license_texts: Vec<&'static str>,
}

impl LicenseData {
    pub fn new(url: &str, license_texts: &[&'static str]) -> Self {
        Self {
            url: url.to_string(),
            license_texts: license_texts.to_vec()
        }
    }
}

struct NoctiG {
    current_page: Page,
    window_time_formatter_index: usize,
    draw_ranges: bool,
    is_showing_help: bool,
    search_text: String,
    search_task_id: String,
    project_creation: Option<ProjectConfiguration>,
    current_project: Option<CurrentProject>,
    recent_projects: Vec<RecentProject>,
    filtered_recent_projects: Option<Vec<RecentProject>>,
    windows: BTreeMap<Id, WindowType>,
    licenses: LazyLock<[Vec<Collapsible<LicenseData>>; 3]>
}

impl CurrentProject {
    pub fn load<P>(path: P) -> Result<Self, Box<dyn Error>> where P : AsRef<Path> {
        let project_xml = fs::read_to_string(&path)?;
        let project = serde_xml_rs::from_str::<Project>(&project_xml)?;
        let project_name = path.as_ref().file_name().map(|p| p.to_string_lossy().to_string()).unwrap_or(String::new());
        let path = path.as_ref().parent().unwrap().to_string_lossy().to_string();

        let readers = project.signals.iter().map(|source| {
            let path = Path::new(&path).join(&source.path);
            let mut reader = EpochReader::new(&path);
            if let Ok(reader) = &mut reader {
                reader.set_start_align_offset(project.epochs_before_current as u64 * EpochReader::EPOCH_DURATION as u64 * 1000);
                reader.set_offset(source.offset);
            }
            reader
        }).collect::<Result<Vec<_>, _>>()?;

        let mut result = Self {
            path,
            project_name,
            readers,
            project,
            markers: Markers::default(),
            annotations: Annotations::default(),
            scorings: None,
            spectrogram: None,
            loading_progress_spectrogram: None
        };

        result.load_labels()?;

        Ok(result)
    }

    pub fn load_labels(&mut self) -> Result<(), Box<dyn Error>>{
        let project_path= Path::new(&self.path);
        let subdir_lables = project_path.join("lables");
        let scores_file = subdir_lables.join("scores.json");
        let markers_file = subdir_lables.join("markers.json");
        let annotations_file = subdir_lables.join("annotations.json");

        // Load stored markers collection file or get default
        self.markers = if markers_file.exists() {
            let markers_json = fs::read_to_string(markers_file)?;
            serde_json::from_str::<Markers>(&markers_json)?
        }
        else {
            Markers::default()
        };

        // Load stored annotations collection file or get default
        self.annotations = if annotations_file.exists() {
            let annotations_json = fs::read_to_string(annotations_file)?;
            serde_json::from_str::<Annotations>(&annotations_json)?
        }
        else {
            Annotations::default()
        };

        // Load stored scores collection file if required for project type
        if self.project.project_type == ProjectType::SleepScoring {
            self.scorings = Some(if scores_file.exists() {
                let scores_json = fs::read_to_string(scores_file)?;
                serde_json::from_str::<Scorings>(&scores_json)?
            }
            else {
                Scorings {
                    epoch_duration: self.project.epoch_duration,
                    values: BTreeMap::new()
                }
            });
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let subdir_lables = Path::new(&self.path).join("lables");
        let scores_file = subdir_lables.join("scores.json");

        // Write current score collection file if required for project type
        if self.project.project_type == ProjectType::SleepScoring {
            let scores_json = serde_json::to_string_pretty(&self.scorings)?;
            fs::write(scores_file, scores_json)?;
        }

        Ok(())
    }
}

impl NoctiG {
    fn boot() -> (NoctiG, Task<Message>) {
        let (_new_id, task) = window::open(WindowType::MainWindow.settings());

        (NoctiG {
            current_page: Page::Home,
            window_time_formatter_index: 1,
            draw_ranges: false,
            is_showing_help: false,
            project_creation: None,
            search_text: String::new(),
            current_project: None,
            recent_projects: Vec::new(),
            search_task_id: String::new(),
            filtered_recent_projects: None,
            windows: BTreeMap::new(),
            licenses: LazyLock::new(|| load_licenses())
        }, Task::batch([
            Task::done(Message::LoadStartPage),
            task.map(move |id| Message::WindowOpened(id, WindowType::MainWindow)),
        ]))
    }

    fn calculate_spectrogram(path: String, source_path: String, signal_index: usize) -> impl Stream<Item = Message> {
        channel(0, move |mut output: Sender<Message>| async move {
            output.send(Message::SpectrogramLoadStart).await.unwrap();

            const SAMPLE_LOAD_PERCENTAGE: f32 = 0.75;

            let mut reader = EDFFile::open(&Path::new(&path).join(&source_path)).unwrap();

            // Get the target signal header
            let signal = reader.header.get_signals().get(signal_index).cloned().unwrap();

            // Get parameters from record and signal headers
            let sf = reader.header.get_signal_sample_frequency(signal_index).unwrap();
            let win_sec = 30.0;
            let nperseg = (win_sec * sf) as i32;
            let record_count = reader.header.get_record_count().unwrap();
            let record_samples = signal.samples_count;
            let sample_count = record_samples * record_count;

            // Collect all samples of the target signal across all records
            let mut i = 0;
            let mut spectro_samples = vec![0.0; sample_count];
            let mut last_progress = 0;
            while let Ok(Some(record)) = reader.read_record() {
                spectro_samples[i * record_samples..(i + 1) * record_samples].copy_from_slice(&record.get_physical_samples(&signal)[signal_index]);
                i += 1;
                let progress = (100.0 * i as f32 / record_count as f32).round() as u16;
                if progress > last_progress {
                    last_progress = progress;
                    output.send(Message::SpectrogramLoadProgress(progress as f32 * SAMPLE_LOAD_PERCENTAGE)).await.unwrap();
                }
            }

            // Calculate the spectrogram from the collected samples
            let spectro_samples = Array1::<f64>::from_vec(spectro_samples);
            let spectrogram = spectrogram_lspopt(spectro_samples, sf, nperseg);

            output.send(Message::SpectrogramLoadFinish(spectrogram)).await.unwrap();
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                return if self.windows.is_empty() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::OpenWindow(window_type) => {
                let Some(last_window) = self.windows.keys().last() else {
                    return Task::none();
                };

                let window_type2 = window_type.clone();
                return window::position(*last_window)
                    .then(move |last_position| {
                        let mut settings = window_type.settings();
                        settings.position = last_position.map_or(Position::Default, |last_position| {
                            Position::Specific(last_position + Vector::new(20.0, 20.0))
                        });
                        let (_, open) = window::open(settings);

                        open
                    })
                    .map(move |id| Message::WindowOpened(id, window_type2.clone()))
            }
            Message::WindowOpened(id, window_type) => {
                self.windows.insert(id, window_type);
            }
            Message::ToggleExpandLicense(category_idx, idx, is_expanded) => {
                self.licenses[category_idx][idx].set_expanded(is_expanded);
            }
            Message::LoadStartPage => {
                if let Ok(recent) = get_recently_opened(25) {
                    self.recent_projects = recent;
                }
            }
            Message::MoveAxis(direction) => {
                if !move_axis(self, direction) {
                    return Task::none();
                }
            },
            Message::Rate(stage) => {
                let Some(project) = &mut self.current_project else {
                    return Task::none();
                };
                let Some(scorings) = &mut project.scorings else {
                    return Task::none();
                };
                if let Some(reader) = project.readers.first() {
                    let current_seg_n = reader.get_window_start_epoch();
                    if stage == Stage::Unset {
                        scorings.values.remove_entry(&current_seg_n);
                    }
                    else {
                        scorings.values.entry(current_seg_n)
                            .and_modify(|v| *v = stage.clone())
                            .or_insert(stage);
                    }
                }
            },
            Message::SpectrogramLoadStart => {

            },
            Message::SpectrogramLoadProgress(progress) => {
                let Some(project) = &mut self.current_project else {
                    return Task::none();
                };
                project.loading_progress_spectrogram = Some(progress);
            },
            Message::SpectrogramLoadFinish(spectrogram) => {
                let Some(project) = &mut self.current_project else {
                    return Task::none();
                };
                project.spectrogram = Some(SpectrogramView::new(spectrogram, "lajolla".to_string()));
            },
            Message::SeekTo => {
                let epoch = 1100;

                let Some(project) = &mut self.current_project else {
                    return Task::none();
                };

                for reader in &mut project.readers {
                    let segment_count = project.project.epochs_before_current as usize + project.project.epochs_after_current as usize + 1;
                    let _ = reader.seek(EpochReader::EPOCH_DURATION as u64 * 1_000 * epoch as u64);
                    reader.read_epochs(segment_count).unwrap();
                }
            },
            Message::CycleTimeFormatter => {
                self.window_time_formatter_index = (self.window_time_formatter_index + 1) % formatting::formatters::TIME_FORMATTERS.len();
            },
            Message::ToggleRangeDraw => {
                self.draw_ranges = !self.draw_ranges;
            },
            Message::ToggleHelp => {
                self.is_showing_help = !self.is_showing_help;
            },
            Message::SwitchPage(page) => {
                self.current_page = page
            },
            Message::CreateProjectWizard => {
                self.project_creation = Some(ProjectConfiguration {
                    name: "".to_string(),
                    path: "".to_string(),
                    new_tag: "".to_string(),
                    tags: Vec::new(),
                    filter_signal: true,
                    auto_align_signals: true,
                    clip_signal: true,
                    data: Vec::new()
                });
                self.current_page = Page::CreateProject(CreatePage::Project);
            },
            Message::SaveProject => {
                let Some(project) = &mut self.current_project else {
                    return Task::none();
                };

                if let Err(e) = project.save() {
                    eprintln!("Error saving project: {}", e);
                    return Task::none()
                };

                println!("SAVED");
            },
            Message::OpenProjectPath(path) => {
                match CurrentProject::load(path) {
                    Ok(project) => self.current_project = Some(project),
                    Err(e) => eprintln!("Error opening project: {}", e) // TODO: Show error message box
                }
                return Task::done(Message::OpenScorer);
            },
            Message::LaunchOpenProject => {
                let path = get_last_project_path()
                    .ok()
                    .flatten()
                    .unwrap_or(String::new());

                return Task::future(async {
                    Message::OpenProject(AsyncFileDialog::new()
                        .add_filter("NoctiG Project", &["ngp"])
                        .set_directory(path)
                        .pick_file()
                        .await
                        .map(|h| h.path().to_path_buf()))
                })
            }
            Message::OpenProject(file) => {
                let Some(file) = file else {
                    return Task::none();
                };

                if let Some(parent) = file.parent() && let Some(path) = parent.to_str() {
                    _ = set_last_project_path(path.to_string());
                }

                let path = file.to_str().map(|p| p.to_string());

                match CurrentProject::load(file) {
                    Ok(project) => {
                        if let Some(path) = path {
                            _ = update_recently_opened(
                                project.project.name.clone(),
                                path.clone()
                            ).unwrap();
                        }
                        self.current_project = Some(project);
                    },
                    Err(e) => eprintln!("Error opening project: {}", e) // TODO: Show error message box
                }

                return Task::done(Message::OpenScorer);
            },
            Message::CancelCreateProject => {
                self.project_creation = None;
                self.current_page = Page::Home;
            },
            Message::CreateProject => {
                // TODO: Require a project name and a location and at least 1 added signal source with at least 1 signal
                //       If not provided, jump to the first erroring page and highlight the field
                if let Some(project) = self.project_creation.take() {
                    return project_initializer::create_new(project);
                }
            },
            Message::OpenScorer => {
                // Move the axis in direction 0, load the data without actually moving on the x-axis
                if !move_axis(self, 0) {
                    return Task::none();
                }

                let Some(project) = &mut self.current_project else {
                    return Task::none();
                };
                project.spectrogram = None;
                project.loading_progress_spectrogram = Some(0.0);
                let source_path = project.project.signals.get(1).unwrap().path.clone();

                // Change the page to the scorer and resize the window
                self.current_page = Page::Scorer;
                return Task::batch([
                    Task::stream(Self::calculate_spectrogram(project.path.clone(), source_path, 0)),
                    resize_window(Size::new(1400.0, 800.0))
                ]);
            },
            Message::CreateProjectWizardError(error) => {
                // TODO: Open dialog box
                eprintln!("{error}");
            },
            Message::ProjectSearchChanged(search) => {
                self.search_text = search.clone();

                // Clear the filter in case the field was cleared
                if search.is_empty() {
                    self.filtered_recent_projects = None;
                    return Task::none();
                }
                let search = search.to_lowercase();

                // Get a somewhat unique identifier for the search to prevent showing results
                // from a previous search that took longer than a subsequent filter
                let search_task_id = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_nanos()
                    .to_string();
                self.search_task_id = search_task_id.clone();
                let recent = self.recent_projects.clone();

                // Perform the filtering asynchronously
                return Task::future(async move {
                    Message::ProjectSearchFiltered(search_task_id, recent.into_iter().filter(|p| {
                        p.name.to_lowercase().contains(&search) || p.path.to_lowercase().contains(&search)
                    }).collect())
                });
            }
            Message::ProjectSearchFiltered(search_task_id, filtered) => {
                if self.search_task_id != search_task_id {
                    return Task::none()
                }

                self.filtered_recent_projects = Some(filtered);
            }
            Message::ToggleFilterSignal(checked) => {
                if let Some(project) = &mut self.project_creation {
                    project.filter_signal = checked;
                }
            },
            Message::ToggleClipSignal(checked) => {
                if let Some(project) = &mut self.project_creation {
                    project.clip_signal = checked;
                }
            },
            Message::ToggleAutoAlignSignals(checked) => {
                if let Some(project) = &mut self.project_creation {
                    project.auto_align_signals = checked;
                }
            },
            Message::NewTagChanged(tag) => {
                if let Some(project) = &mut self.project_creation {
                    project.new_tag = tag;
                }
            },
            Message::ProjectLocationChanged(path) => {
                if let Some(project) = &mut self.project_creation {
                    project.path = path;
                }
            },
            Message::ProjectNameChanged(name) => {
                if let Some(project) = &mut self.project_creation {
                    project.name = name;
                }
            },
            Message::AddTag => {
                if let Some(project) = &mut self.project_creation {
                    if !project.new_tag.trim().is_empty() {
                        project.tags.push(project.new_tag.clone());
                        project.new_tag = String::new();
                        // TODO: Focus tags input field again
                    }
                }
            },
            Message::RemoveTag(index) => {
                if let Some(project) = &mut self.project_creation {
                    project.tags.remove(index);
                }
            },
            Message::LaunchBrowseProjectLocation => {
                let path = get_last_project_create_path()
                    .ok()
                    .flatten()
                    .unwrap_or(String::new());

                return Task::future(async {
                    Message::BrowseProjectLocation(AsyncFileDialog::new()
                        .set_directory(path)
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf())
                    )
                });
            }
            Message::BrowseProjectLocation(folder) => {
                let Some(path) = folder.map(|p| p.to_str().map(|v| v.to_string())).flatten() else {
                    return Task::none();
                };

                _ = set_last_project_create_path(path.to_string());

                if let Some(project) = &mut self.project_creation {
                    project.path = path.to_string();
                }
            },
            Message::LaunchBrowseImportSignal => {
                let path = get_last_browse_source_path()
                    .ok()
                    .flatten()
                    .unwrap_or(String::new());

                return Task::future(async {
                    Message::BrowseImportSignal(AsyncFileDialog::new()
                        .add_filter("EDF/EDF+ File", &["edf"])
                        .set_directory(path)
                        .pick_files()
                        .await
                        .map(|h| h.iter().map(|h| h.path().to_path_buf()).collect())
                    )
                });
            }
            Message::BrowseImportSignal(files) => {
                if let Some(files) = files {
                    if let Some(first) = files.first() && let Some(parent) = first.parent() && let Some(path) = parent.to_str() {
                        _ = set_last_browse_source_path(path.to_string());
                    };

                    if let Some(project) = &mut self.project_creation {
                        // TODO: Skip all files which are already present in the added data (and maybe also check for duplicates in current list
                        //       which would probably be useless as you most likely cannot select a file twice)
                        let signals = files.iter().filter_map(|path| {
                            path.to_str().map(|s| (EDFFile::open(s.to_string()).ok(), s.to_string()))
                        }).map(|(edf, path)| {
                            let mut duration = 0.0;
                            let mut signal_count = 0;
                            let mut timestamp = 0;

                            if let Some(edf) = edf {
                                let header = edf.header;
                                duration = header.get_record_count().map(|c| c as f64 * header.get_record_duration()).unwrap_or(0.0);
                                signal_count = header.get_signals().len();
                                timestamp = header.start_date().and_time(header.get_start_time()).and_utc().timestamp() as u64;
                            };

                            // TODO: In case there already is a file with this name in the current signals, append a -<NUMERIC> to make it unique
                            let filename = Path::new(&path).file_name()
                                .map(|name| name.to_string_lossy().to_string())
                                .unwrap_or("--".to_string());

                            ProjectSignals {
                                timestamp,
                                duration,
                                signal_count,
                                path,
                                name: filename,
                                is_reference: false
                            }
                        });
                        project.data.append(&mut signals.collect());
                    }
                }
            },
            Message::RemoveImportSignal(path) => {
                if let Some(project) = &mut self.project_creation {
                    if let Some(index) = project.data.iter().position(|signal| signal.path == path) {
                        project.data.remove(index);
                    };
                }
            },
            Message::ShowSourceCode => {
                if let Err(error) = webbrowser::open("https://github.com/BitFlaker/noctig-scorer") {
                    warn!("Error opening source code in default browser: {}", error);
                }
            },
            Message::OpenURL(url) => {
                if let Err(error) = webbrowser::open(&url) {
                    warn!("Error opening url {} in default browser: {}", url, error);
                }
            },
            Message::ShowPrivacyPolicy => {
                panic!("NYI")
            }
        }

        Task::none()
    }

    fn view(&self, id: Id) -> Element<'_, Message> {
        let Some(window_type) = self.windows.get(&id) else {
            return space().into();
        };

        match window_type {
            WindowType::MainWindow => {
                match self.current_page {
                    Page::Home | Page::Stream | Page::Help | Page::Settings => start::view(self),
                    Page::Scorer => scorer::view(self),
                    Page::CreateProject(ref page) => create_viewer::view(self, page),
                }
            }
            WindowType::Licenses => {
                license::view(self)
            }
            WindowType::About => {
                space().into()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::batch([
            window::close_events().map(Message::WindowClosed),
            event::listen_with(|event, status, _| match (event, status) {
                (
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: Key::Named(Named::ArrowRight),
                        ..
                    }),
                    Status::Ignored,
                ) => Some(Message::MoveAxis(1)),
                (
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: Key::Named(Named::ArrowLeft),
                        ..
                    }),
                    Status::Ignored,
                ) => Some(Message::MoveAxis(-1)),
                (
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: Key::Named(Named::Delete),
                        ..
                    }),
                    Status::Ignored,
                ) => Some(Message::Rate(Stage::Unset)),
                (
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: Key::Character(k),
                        modifiers,
                        ..
                    }),
                    Status::Ignored,
                ) => match k.to_string().to_lowercase().as_str() {
                    "w" => Some(Message::Rate(Stage::W)),
                    "r" => Some(Message::Rate(Stage::R)),
                    "1" => Some(Message::Rate(Stage::N1)),
                    "2" => Some(Message::Rate(Stage::N2)),
                    "3" => Some(Message::Rate(Stage::N3)),
                    "t" => Some(Message::CycleTimeFormatter),
                    "l" => Some(Message::ToggleRangeDraw),
                    "h" => Some(Message::ToggleHelp),
                    "j" => Some(Message::SeekTo),
                    "s" if modifiers.control() => Some(Message::SaveProject),
                    _ => None
                },
                _ => None,
            })
        ])
    }

    pub fn theme<'a>(&'a self, _id: Id) -> Option<Theme> {
        Some(
            Theme::custom_with_fn(
                "ClearDark".to_string(),
                formatting::theme::CLEAR_DARK,
                formatting::theme::generate_extended
            )
        )
    }
}

fn resize_window(new_size: Size) -> Task<Message> {
    // TODO: Show some transition page which does not have scaling artifacts

    window::oldest().and_then(move |id| {
        window::size(id).then(move |old_size| {
            window::position(id).then(move |old_position| {
                if let Some(old_position) = old_position {
                    let diff = old_size - new_size;
                    let x = old_position.x + diff.width / 2.0;
                    let y = old_position.y + diff.height / 2.0;
                    return Task::batch([
                        window::resize::<Message>(id, new_size),
                        window::move_to(id, Point::new(x, y)),
                    ]);
                }

                window::resize::<Message>(id, new_size)
            })
        })
    })
}

fn move_axis(app: &mut NoctiG, direction: i8) -> bool {
    let Some(project) = &mut app.current_project else {
        return false;
    };

    // Ensure not to surpass the last possible epoch across all readers
    let max_epoch_reader = project.readers.iter().max_by(|r1, r2| r1.get_epoch_count().cmp(&r2.get_epoch_count())).unwrap();
    let max_epoch = max_epoch_reader.get_epoch_count();
    let current_epoch = max_epoch_reader.get_window_start_epoch();

    if current_epoch == max_epoch - 1 && direction == 1 {
        return false;
    }

    // Move and read all visible samples
    for reader in &mut project.readers {
        let segment_count = project.project.epochs_before_current as usize + project.project.epochs_after_current as usize + 1;
        seek_segmented(reader, segment_count, direction);
    }

    true
}

fn seek_segmented(reader: &mut EpochReader, segment_count: usize, direction: i8) {
    let _ = reader.seek(u64::try_from(reader.tell() - (EpochReader::EPOCH_DURATION as i128 * 1_000 * (segment_count as i128 - direction as i128))).unwrap_or(0));
    reader.read_epochs(segment_count).unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub enum Marker {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Stage {
    W,
    N1,
    N2,
    N3,
    R,
    Unset
}

#[derive(Debug, Clone, PartialEq)]
enum Page {
    Home,
    Stream,
    Help,
    CreateProject(CreatePage),
    Settings,
    Scorer,
}

#[derive(Debug, Clone, PartialEq)]
enum CreatePage {
    Project,
    Data,
    Processing
}

impl Stage {
    pub fn map_str(&self) -> String {
        match self {
            Stage::W => "W".to_string(),
            Stage::N1 => "N1".to_string(),
            Stage::N2 => "N2".to_string(),
            Stage::N3 => "N3".to_string(),
            Stage::R => "R".to_string(),
            Stage::Unset => "?".to_string()
        }
    }

    pub fn background(stage: Stage) -> impl Fn(&Theme) -> widget::container::Style {
        move |theme: &Theme| border_background_base(theme, &stage)
    }

    pub fn foreground(stage: Stage) -> impl Fn(&Theme) -> widget::text::Style {
        move |theme: &Theme| text_foreground_base(theme, &stage)
    }
}

#[derive(Clone)]
enum Message {
    MoveAxis(i8),
    Rate(Stage),
    SpectrogramLoadStart,
    SpectrogramLoadProgress(f32),
    SpectrogramLoadFinish(Spectrogram),
    CycleTimeFormatter,
    ToggleRangeDraw,
    ToggleHelp,
    SeekTo,
    SaveProject,
    SwitchPage(Page),
    WindowClosed(Id),
    ToggleExpandLicense(usize, usize, bool),

    OpenScorer,

    // Start page
    LoadStartPage,
    ProjectSearchChanged(String),
    ProjectSearchFiltered(String, Vec<RecentProject>),
    CreateProjectWizard,
    CreateProjectWizardError(String),
    OpenProjectPath(String),
    LaunchOpenProject,
    OpenProject(Option<PathBuf>),
    ShowSourceCode,
    OpenURL(String),
    ShowPrivacyPolicy,
    OpenWindow(WindowType),
    WindowOpened(Id, WindowType),

    // Project Creation Wizard
    CancelCreateProject,
    CreateProject,
    ProjectNameChanged(String),
    ProjectLocationChanged(String),
    NewTagChanged(String),
    AddTag,
    RemoveTag(usize),
    LaunchBrowseProjectLocation,
    BrowseProjectLocation(Option<PathBuf>),
    LaunchBrowseImportSignal,
    BrowseImportSignal(Option<Vec<PathBuf>>),
    RemoveImportSignal(String),
    ToggleFilterSignal(bool),
    ToggleClipSignal(bool),
    ToggleAutoAlignSignals(bool)
}
