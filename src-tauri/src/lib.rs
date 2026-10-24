mod commands;
mod error;
mod requisitions;
mod settings;
mod state;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let settings_path = config_dir.join(settings::SETTINGS_FILE_NAME);
            let app_settings = match tauri::async_runtime::block_on(settings::load(&settings_path))
            {
                Ok(app_settings) => app_settings,
                Err(err) => {
                    error::dev_log(&format!("settings load failed, using defaults: {err}"));
                    settings::AppSettings::default()
                }
            };
            let requisitions_path = config_dir.join(requisitions::REQUISITIONS_FILE_NAME);
            let records =
                match tauri::async_runtime::block_on(requisitions::load(&requisitions_path)) {
                    Ok(records) => records,
                    Err(err) => {
                        error::dev_log(&format!("requisition store load failed: {err}"));
                        let backup = config_dir.join(format!(
                            "{}.corrupt-{}.json",
                            requisitions::REQUISITIONS_FILE_NAME,
                            chrono::Local::now().format("%Y%m%d%H%M%S")
                        ));
                        if let Err(rename_err) = std::fs::rename(&requisitions_path, &backup) {
                            error::dev_log(&format!(
                                "could not back up the corrupt requisition store: {rename_err}"
                            ));
                        }
                        Vec::new()
                    }
                };

            let state = AppState::new(config_dir, app_settings, records);
            let monitor_state = state.clone();
            tauri::async_runtime::spawn(async move {
                commands::run_health_monitor(monitor_state).await;
            });
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connection_status,
            commands::connection_health,
            commands::connection_summary,
            commands::configure_connection,
            commands::connect,
            commands::reconnect,
            commands::login,
            commands::logout,
            commands::current_session,
            commands::test_connection,
            commands::load_settings,
            commands::save_doctors,
            commands::save_catalog,
            commands::save_report,
            commands::save_approvals,
            commands::export_settings_to_file,
            commands::search_drugs,
            commands::search_doctors,
            commands::fetch_usage,
            commands::period_context,
            commands::list_requisitions,
            commands::save_and_lock_requisition,
        ])
        .run(tauri::generate_context!())
        .expect(
            "invariant: tauri::run() fails only when the platform cannot launch the app window",
        );
}
