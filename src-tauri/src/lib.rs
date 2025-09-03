use std::fs;
use serde::Serialize;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[derive(Debug,Serialize)]
pub struct HostEntry {
    ip: String,
    hostname: String,
    enabled: bool
}

#[tauri::command]
fn backup_hosts() -> Result<String,String> {
    let hosts_path = default_hosts_path();

    let desk_top = dirs::desktop_dir().ok_or("无法获取桌面目录")?;

    let bak_path = desk_top.with_file_name(format!(
        "hosts.bak.{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    ));

    fs::copy(&hosts_path,&bak_path)
        .map_err(|e| format!("备份错误：{}",e))?;

    Ok(bak_path.to_string_lossy().to_string())
}

#[tauri::command]
fn load_hosts() -> Vec<HostEntry> {
    let hosts_path = default_hosts_path();

    read_file(&hosts_path).unwrap_or_else(|e| {
        eprintln!("读取文件错误: {}", e);
        vec![]
    })
}
fn read_file(path: &PathBuf) -> Result<Vec<HostEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    let mut res = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let line = line.trim();

        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 按空白切分
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue; // 格式不正确，跳过
        }

        let ip = parts[0].to_string();
        // 一行可能有多个 hostname
        for host in &parts[1..] {
            res.push(HostEntry {
                ip: ip.clone(),
                hostname: host.to_string(),
                enabled: true,
            });
        }
    }

    Ok(res)
}

/// 根据操作系统选择 hosts 文件路径
fn default_hosts_path() -> PathBuf {
    use std::path::PathBuf;
    use std::env;

    if cfg!(target_os = "windows") {
        let sys_root = env::var("SystemRoot").unwrap_or("C:\\Windows".to_string());
        PathBuf::from(sys_root).join("System32").join("drivers").join("etc").join("hosts")
    }else {
        PathBuf::from("/etc/hosts")
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_hosts, backup_hosts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
