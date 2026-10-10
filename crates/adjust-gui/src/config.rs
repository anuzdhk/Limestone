use crate::theme::{AccentTheme, ThemeMode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum AppLanguage {
    English,
    Spanish,
    French,
    German,
    Japanese,
}

impl AppLanguage {
    pub fn name(&self) -> &'static str {
        match self {
            AppLanguage::English => "English",
            AppLanguage::Spanish => "Español",
            AppLanguage::French => "Français",
            AppLanguage::German => "Deutsch",
            AppLanguage::Japanese => "日本語",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub theme_mode: ThemeMode,
    pub accent_theme: AccentTheme,
    pub ui_scale: f32,
    pub language: AppLanguage,
    pub gpu_acceleration: bool,
    pub high_quality_tiled_preview: bool,
    pub recent_files: Vec<PathBuf>,
    pub favorite_presets: HashSet<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::DarkStudio,
            accent_theme: AccentTheme::ElectricCyan,
            ui_scale: 1.0,
            language: AppLanguage::English,
            gpu_acceleration: true,
            high_quality_tiled_preview: true,
            recent_files: Vec::new(),
            favorite_presets: HashSet::new(),
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("limestone_pro_settings.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, data);
        }
    }

    pub fn add_recent(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
        self.save();
    }
}
