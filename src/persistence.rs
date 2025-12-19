use crate::data::ProjectData;
use std::fs;
use std::path::Path;

pub struct PersistencePlugin;

impl bevy::prelude::Plugin for PersistencePlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
}

pub fn save_project(project: &ProjectData, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(project)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_project(path: &str) -> std::io::Result<ProjectData> {
    if !Path::new(path).exists() {
        return Ok(ProjectData::default());
    }
    let data = fs::read_to_string(path)?;
    let project: ProjectData = serde_json::from_str(&data)?;
    Ok(project)
}
