use crate::theme::{Accent, ThemeMode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Per-user application data folder:
/// `%APPDATA%\LimeStone` on Windows, `~/Library/Application Support/LimeStone` on macOS,
/// `$XDG_CONFIG_HOME/limestone` (or `~/.config/limestone`) elsewhere.
pub fn app_data_dir() -> PathBuf {
    let dir = if cfg!(windows) {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("LimeStone"))
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/LimeStone"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("limestone"))
    };
    let dir = dir.unwrap_or_else(|| std::env::temp_dir().join("LimeStone"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub theme_mode: ThemeMode,
    pub accent: Accent,
    pub ui_scale: f32,
    pub high_quality_tiled_preview: bool,
    pub show_sidebar: bool,
    pub show_inspector: bool,
    pub recent_files: Vec<PathBuf>,
    pub favorite_presets: HashSet<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            accent: Accent::Blue,
            ui_scale: 1.0,
            high_quality_tiled_preview: true,
            show_sidebar: true,
            show_inspector: true,
            recent_files: Vec::new(),
            favorite_presets: HashSet::new(),
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        app_data_dir().join("settings.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::config_path(), data);
        }
    }

    pub fn add_recent(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(10);
        self.save();
    }
}
