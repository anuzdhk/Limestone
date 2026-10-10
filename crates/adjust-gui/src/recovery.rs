use adjust_core::AdjustParams;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct RecoveryData {
    pub image_path: Option<PathBuf>,
    pub params: AdjustParams,
    pub timestamp: u64,
}

pub struct AutoSaveManager {
    file_path: PathBuf,
    last_saved_params: Option<AdjustParams>,
}

impl AutoSaveManager {
    pub fn new() -> Self {
        let mut path = std::env::temp_dir();
        path.push("limestone_pro_recovery.json");
        Self {
            file_path: path,
            last_saved_params: None,
        }
    }

    pub fn auto_save(&mut self, image_path: Option<&PathBuf>, params: &AdjustParams) {
        if let Some(ref last) = self.last_saved_params {
            if last == params {
                return;
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let data = RecoveryData {
            image_path: image_path.cloned(),
            params: params.clone(),
            timestamp,
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = std::fs::write(&self.file_path, json);
            self.last_saved_params = Some(params.clone());
        }
    }

    pub fn try_recover() -> Option<RecoveryData> {
        let mut path = std::env::temp_dir();
        path.push("limestone_pro_recovery.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<RecoveryData>(&content) {
                    return Some(data);
                }
            }
        }
        None
    }

    pub fn clear_recovery(&self) {
        let _ = std::fs::remove_file(&self.file_path);
    }
}
