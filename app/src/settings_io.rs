use serde::{Deserialize, Serialize};
use snap_herb_models::AppSettings;

pub const EXPORT_FORMAT: &str = "snapherb-settings";
pub const EXPORT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsExport {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub settings: AppSettings,
}

pub fn export_json(settings: &AppSettings) -> Result<String, String> {
    let file = SettingsExport {
        format: EXPORT_FORMAT.to_string(),
        version: EXPORT_VERSION,
        exported_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        settings: settings.clone(),
    };
    serde_json::to_string_pretty(&file).map_err(|_| "สร้างไฟล์ตั้งค่าไม่สำเร็จ".to_string())
}

pub fn parse_import(raw: &str) -> Result<AppSettings, String> {
    let file: SettingsExport = serde_json::from_str(raw)
        .map_err(|_| "ไฟล์ไม่ถูกต้อง: ไม่ใช่ไฟล์ตั้งค่าของ SnapHerb หรือไฟล์เสียหาย".to_string())?;
    if file.format != EXPORT_FORMAT {
        return Err("ไฟล์ไม่ถูกต้อง: ไม่ใช่ไฟล์ตั้งค่าของ SnapHerb".to_string());
    }
    if file.version > EXPORT_VERSION {
        return Err("ไฟล์ถูกสร้างจากเวอร์ชันใหม่กว่า กรุณาอัปเดตโปรแกรม".to_string());
    }
    Ok(file.settings)
}

pub fn export_file_name() -> String {
    format!(
        "snapherb-settings-{}.json",
        chrono::Local::now().format("%Y%m%d")
    )
}

pub fn download_json(json: &str, file_name: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or_else(|| "ไม่พบหน้าต่างโปรแกรม".to_string())?;
    let document = window
        .document()
        .ok_or_else(|| "ไม่พบเอกสารของโปรแกรม".to_string())?;

    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(json));
    let blob = web_sys::Blob::new_with_str_sequence(&parts)
        .map_err(|_| "สร้างไฟล์สำหรับดาวน์โหลดไม่สำเร็จ".to_string())?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "สร้างไฟล์สำหรับดาวน์โหลดไม่สำเร็จ".to_string())?;

    let anchor = document
        .create_element("a")
        .map_err(|_| "สร้างไฟล์สำหรับดาวน์โหลดไม่สำเร็จ".to_string())?
        .unchecked_into::<web_sys::HtmlAnchorElement>();
    anchor.set_href(&url);
    anchor.set_download(file_name);
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_round_trips() {
        let json = export_json(&AppSettings::default()).expect("export");
        assert!(json.contains("snapherb-settings"));
        let parsed = parse_import(&json).expect("import");
        assert_eq!(parsed, AppSettings::default());
    }

    #[test]
    fn foreign_json_is_rejected() {
        let error = parse_import("{\"hello\":\"world\"}").expect_err("reject");
        assert!(error.contains("SnapHerb"));
    }

    #[test]
    fn newer_version_is_rejected() {
        let json = r#"{
            "format": "snapherb-settings",
            "version": 99,
            "exportedAt": "2026-10-01 08:00",
            "settings": {}
        }"#;
        let error = parse_import(json).expect_err("reject");
        assert!(error.contains("เวอร์ชันใหม่กว่า"));
    }

    #[test]
    fn export_file_name_is_json() {
        let name = export_file_name();
        assert!(name.starts_with("snapherb-settings-"));
        assert!(name.ends_with(".json"));
    }
}
