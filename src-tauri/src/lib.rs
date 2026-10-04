use tauri_plugin_opener::OpenerExt;

/// `autoplay=1` because the whole point is to give the user no choice.
const RICKROLL: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&autoplay=1";

/// Open the video in the system default browser.
///
/// Deliberately called from Rust rather than from Angular: the JS `openUrl` command is
/// gated by the opener plugin's scope ACL, while the `Opener` methods are not. No
/// capability or scope widening is needed to reach YouTube.
#[tauri::command]
fn rickroll(app: tauri::AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(RICKROLL, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Fires as the app starts, before the window paints. Browsers block audible
        // autoplay without a user gesture, so the tab can land paused - it is still
        // in front of the user. The button in the window re-rolls on demand.
        .setup(|app| {
            if let Err(e) = rickroll(app.handle().clone()) {
                eprintln!("rickroll: could not open the browser: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![rickroll])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
