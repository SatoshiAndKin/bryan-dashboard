use crate::model::settings::AppSettings;
use crate::model::WorkbookState;

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY: &str = "bd.workbook.v1";

#[cfg(target_arch = "wasm32")]
const SETTINGS_KEY: &str = "bd.settings.v1";

pub fn load_workbook() -> WorkbookState {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(json) = read_local_storage(STORAGE_KEY) {
            match serde_json::from_str::<WorkbookState>(&json) {
                Ok(mut wb) => {
                    wb.migrate_if_needed();
                    return wb;
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to deserialize workbook: {e}").into(),
                    );
                }
            }
        }
    }
    WorkbookState::default()
}

pub fn save_workbook(wb: &WorkbookState) {
    #[cfg(target_arch = "wasm32")]
    {
        match serde_json::to_string(wb) {
            Ok(json) => write_local_storage(STORAGE_KEY, &json),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to serialize workbook: {e}").into());
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = wb;
    }
}

pub fn load_settings() -> AppSettings {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(json) = read_local_storage(SETTINGS_KEY) {
            match serde_json::from_str::<AppSettings>(&json) {
                Ok(s) => return s,
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to deserialize settings: {e}").into(),
                    );
                }
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) {
    #[cfg(target_arch = "wasm32")]
    {
        match serde_json::to_string(settings) {
            Ok(json) => write_local_storage(SETTINGS_KEY, &json),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to serialize settings: {e}").into());
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = settings;
    }
}

pub fn export_workbook(wb: &WorkbookState) -> Option<String> {
    serde_json::to_string_pretty(wb).ok()
}

pub fn import_workbook(json: &str) -> Result<WorkbookState, String> {
    serde_json::from_str::<WorkbookState>(json).map_err(|e| format!("Import failed: {e}"))
}

#[cfg(target_arch = "wasm32")]
fn read_local_storage(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

#[cfg(target_arch = "wasm32")]
fn write_local_storage(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}
