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

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn export_workbook(wb: &WorkbookState) -> Option<String> {
    serde_json::to_string_pretty(wb).ok()
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let mut wb = WorkbookState::default();
        if let Some(sheet) = wb.active_sheet_mut() {
            if let Some(t) = sheet.active_table_mut() {
                t.set_cell_source(0, 0, "Hello".to_string());
                t.set_cell_source(1, 0, "42".to_string());
                t.set_cell_source(0, 1, "=A1".to_string());
            }
        }

        let json = export_workbook(&wb).expect("export should succeed");
        let wb2 = import_workbook(&json).expect("import should succeed");

        assert_eq!(wb.version, wb2.version);
        assert_eq!(wb.sheets.len(), wb2.sheets.len());
        let t1 = wb.active_sheet().unwrap().active_table().unwrap();
        let t2 = wb2.active_sheet().unwrap().active_table().unwrap();
        assert_eq!(t1.cells[&(0, 0)].source, t2.cells[&(0, 0)].source);
        assert_eq!(t1.cells[&(1, 0)].source, t2.cells[&(1, 0)].source);
        assert_eq!(t1.cells[&(0, 1)].source, t2.cells[&(0, 1)].source);
    }

    #[test]
    fn test_import_invalid_json() {
        let result = import_workbook("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_wrong_structure() {
        let result = import_workbook(r#"{"foo": "bar"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_produces_valid_json() {
        let wb = WorkbookState::default();
        let json = export_workbook(&wb).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("version").is_some());
        assert!(parsed.get("sheets").is_some());
    }

    #[test]
    fn test_load_workbook_returns_default_on_native() {
        let wb = load_workbook();
        assert_eq!(wb.version, 2);
        assert_eq!(wb.sheets.len(), 1);
    }

    #[test]
    fn test_load_settings_returns_default_on_native() {
        let s = load_settings();
        assert_eq!(s.poll_interval_secs, 10);
    }
}
