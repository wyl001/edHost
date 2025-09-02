use serde::Serialize;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[derive(Serialize)]
pub struct HostEntry {
    ip: String,
    hostname: String,
    enabled: bool
}


#[tauri::command]
fn load_hosts() ->Vec<HostEntry> {

}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_hosts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
