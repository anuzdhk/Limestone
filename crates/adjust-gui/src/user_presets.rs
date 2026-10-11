//! "My Collection": user presets stored as JSON files in the app data folder.

use crate::config::app_data_dir;
use adjust_core::{AdjustParams, Preset};
use std::path::{Path, PathBuf};

pub const MY_COLLECTION: &str = "My Collection";

fn presets_dir() -> PathBuf {
    let dir = app_data_dir().join("presets");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Turns a preset name into a safe file name.
fn file_for(name: &str) -> PathBuf {
    let safe: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || " -_".contains(c) { c } else { '_' })
        .collect();
    presets_dir().join(format!("{}.json", safe.trim()))
}

/// Loads every user preset, sorted by name. Unreadable files are skipped.
pub fn load_all() -> Vec<Preset> {
    let mut out: Vec<Preset> = std::fs::read_dir(presets_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|json| Preset::from_json(&json).ok())
        .map(|mut p| {
            p.category = MY_COLLECTION.to_string();
            p.params = p.params.sanitized();
            p
        })
        .collect();
    out.sort_by_key(|p| p.name.to_lowercase());
    out
}

pub fn save(name: &str, params: &AdjustParams) -> Result<Preset, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Preset name cannot be empty".into());
    }
    let preset = Preset::new(name, MY_COLLECTION, "Saved from your adjustments", params.clone());
    let json = preset.to_json().map_err(|e| e.to_string())?;
    std::fs::write(file_for(name), json).map_err(|e| e.to_string())?;
    Ok(preset)
}

pub fn delete(name: &str) -> Result<(), String> {
    std::fs::remove_file(file_for(name)).map_err(|e| e.to_string())
}

/// Imports a preset JSON file into My Collection.
pub fn import(path: &Path) -> Result<Preset, String> {
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let preset = Preset::from_json(&json).map_err(|e| format!("Invalid preset file: {e}"))?;
    save(&preset.name, &preset.params.sanitized())
}

/// Writes any preset to a user-chosen path.
pub fn export(preset: &Preset, path: &Path) -> Result<(), String> {
    let json = preset.to_json().map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}
