use adjust_core::AdjustParams;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct RecoveryData {
    pub image_path: Option<PathBuf>,
    /// Applied ("baked") layers, oldest first.
    #[serde(default)]
    pub layers: Vec<AdjustParams>,
    pub params: AdjustParams,
    pub timestamp: u64,
}

pub struct AutoSaveManager {
    file_path: PathBuf,
    last_saved: Option<(Vec<AdjustParams>, AdjustParams)>,
}

impl AutoSaveManager {
    pub fn new() -> Self {
        let mut path = std::env::temp_dir();
        path.push("limestone_pro_recovery.json");
        Self {
            file_path: path,
            last_saved: None,
        }
    }

    pub fn auto_save(
        &mut self,
        image_path: Option<&PathBuf>,
        layers: &[AdjustParams],
        params: &AdjustParams,
    ) {
        if let Some((ref last_layers, ref last_params)) = self.last_saved {
            if last_layers == layers && last_params == params {
                return;
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let data = RecoveryData {
            image_path: image_path.cloned(),
            layers: layers.to_vec(),
            params: params.clone(),
            timestamp,
        };

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = std::fs::write(&self.file_path, json);
            self.last_saved = Some((layers.to_vec(), params.clone()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_recovery_files_without_layers_still_load() {
        let json = r#"{"image_path":null,"params":{},"timestamp":1}"#;
        let data: RecoveryData = serde_json::from_str(json).unwrap();
        assert!(data.layers.is_empty());
        assert_eq!(data.params, AdjustParams::default());
    }
}
