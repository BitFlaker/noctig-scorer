use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::error::Error;
use std::collections::BTreeMap;
use std::fs::{create_dir_all, read_dir};

use iced::Task;
use xml::EmitterConfig;
use serde_xml_rs::SerdeXml;

use crate::{Annotations, Markers, Message, Project, ProjectConfiguration, ProjectType, Scorings, SessionState};

const ILLEGAL_PATH_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

// The `edfplus` crate requires EDF files to have a value starting with `EDF+C` in the header of 
// the EDF file at position 192 to 236. Therefore some valid EDF files might be considered invalid.
// With this flag set to `true`, the header will be overwritten at this position to ensure the value 
// to start with `EDF+C`. This is a workaround which is hopefully not required in the future.
const ENSURE_EDFPLUS_TAG_INSERTED: bool = true;

pub fn create_new(project: ProjectConfiguration) -> Task<Message> {
    Task::future(create_new_handler(project))
}

async fn create_new_handler(project: ProjectConfiguration) -> Message {
    match create_new_async(project).await {
        Ok(path) => Message::OpenProjectPath(path),
        Err(e) => Message::CreateProjectWizardError(e.to_string())
    }
}

async fn create_new_async(config: ProjectConfiguration) -> Result<String, Box<dyn Error>> {
    let project_name = sanitize_file_name(&config.name);
    let project_path = Path::new(&config.path).join(&project_name);
    let project_file = project_path.join(format!("{}.ngp", project_name));
    let subdir_sources = project_path.join("sources");
    let subdir_processed = project_path.join("processed");
    let subdir_lables = project_path.join("lables");
    let scores_file = subdir_lables.join("scores.json");
    let markers_file = subdir_lables.join("markers.json");
    let annotations_file = subdir_lables.join("annotations.json");
    let session_file = project_path.join("session.json");

    // Create the directory if it is missing and throw an error
    // in case the directory exists and is not empty
    if project_path.is_dir() {
        if read_dir(project_path).is_ok_and(|r| r.count() > 0) {
            return Err("Project directory already exists and is not empty".into());
        }
    }
    else {
        create_dir_all(project_path)?;
    }
    
    // Create and store the project file
    let xml_serializer = SerdeXml::new().emitter(EmitterConfig::new().perform_indent(true));
    let project = Project::from_config(&config);
    let xml = xml_serializer.to_string(&project)?;
    fs::write(&project_file, xml)?;

    // Create project directory structure
    create_dir_all(&subdir_sources)?;
    create_dir_all(subdir_processed)?;
    create_dir_all(&subdir_lables)?;

    // Import or reference the signals
    for source in config.data.iter().filter(|s| !s.is_reference) {
        let target = subdir_sources.join(&source.name);
        fs::copy(&source.path, &target)?;

        // Workaround to be able to try read any EDF file
        if ENSURE_EDFPLUS_TAG_INSERTED {
            let mut file = OpenOptions::new()
                .write(true)
                .open(target)?;
            file.seek(SeekFrom::Start(192))?;
            file.write_all(b"EDF+C")?;
        }
    }

    // TODO: Perform MNE filter on all EDF files in the 'source' directory and write filtered data in place

    // Create the default markers collection file
    let markers_json = serde_json::to_string_pretty(&Markers::default())?;
    fs::write(markers_file, markers_json)?;

    // Create the default annotations collection file
    let annotations_json = serde_json::to_string_pretty(&Annotations::default())?;
    fs::write(annotations_file, annotations_json)?;

    // Create the default scores collection file if required for project type
    if project.project_type == ProjectType::SleepScoring {
        let scores_json = serde_json::to_string_pretty(&Scorings { 
            epoch_duration: project.epoch_duration,
            values: BTreeMap::new()
        })?;
        fs::write(scores_file, scores_json)?;
    }

    // Create the default user session state
    let session_json = serde_json::to_string_pretty(&SessionState::default())?;
    fs::write(session_file, session_json)?;

    Ok(project_file.to_string_lossy().to_string())
}

pub fn sanitize_file_name(value: &str) -> String {
    let positions = illegal_path_char_positions(value);
    value.char_indices()
        .filter_map(|(i, c)| if positions.contains(&i) { None } else { Some(c) })
        .collect::<String>()
}

pub fn illegal_path_char_positions(value: &str) -> Vec<usize> {
    value.char_indices()
        .filter_map(|(i, c)| 
            if ILLEGAL_PATH_CHARS.contains(&c) || c <= '\u{1F}' { 
                Some(i) 
            } else { 
                None 
            })
        .collect()
}