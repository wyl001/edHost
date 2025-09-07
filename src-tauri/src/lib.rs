use std::fs;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;

#[derive(Debug,Serialize,Deserialize)]
pub struct HostEntry {
    ip: String,
    hostname: String,
    enabled: bool
}

#[derive(Debug,Serialize,Deserialize)]
pub struct HostForm {
    ip: String,
    hostname: String
}

#[tauri::command]
fn open_file() {
    #[cfg(target_os = "windows")] {
        std::process::Command::new("notepad")
            .arg("C:\\Windows\\System32\\drivers\\etc\\hosts")
            .spawn()
            .unwrap();
    }
    #[cfg(target_os = "macos")] {
        std::process::Command::new("open")
            .arg("-e")
            .arg("/etc/hosts")
            .spawn()
            .unwrap();
    }
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

#[tauri::command]
fn add_host_entry(host_form: HostForm) -> Result<(), String> {
    let entries = load_hosts();
    
    // 检查是否已存在相同的映射
    if entries.iter().any(|e| e.ip == host_form.ip && e.hostname == host_form.hostname) {
        return Err(format!("该映射已存在：ip->{},hostName->{}", host_form.ip, host_form.hostname));
    }
    let new_data= vec![HostEntry {
        ip: host_form.ip.clone(),
        hostname: host_form.hostname.clone(),
        enabled: true,
    }];

    let result = save_hosts_internal(&new_data);
    result
}

#[tauri::command]
fn save_hosts(entries: Vec<HostEntry>) -> Result<(), String> {
    let hosts_path = default_hosts_path();

    let mut new_content = String::new();
    new_content.push_str("# Hosts file managed by edHost\n");
    new_content.push_str("# Generated automatically\n\n");

    for entry in entries {
        if entry.enabled {
            new_content.push_str(format!("{} {} \n", entry.ip, entry.hostname).as_str());
        } else {
            new_content.push_str(format!("# {} {} \n", entry.ip, entry.hostname).as_str());
        }
    }
    println!("{}", new_content);

    // 写入文件
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)  // 清空文件内容再写入
        .open(&hosts_path) // 将 String 转为 &str
        .map_err(|e| format!("无法打开文件: {}", e))?;

    file.write_all(new_content.as_bytes())
        .map_err(|e| format!("写入文件错误: {}", e))?;

    Ok(())
}

fn save_hosts_internal(entries: &[HostEntry]) -> Result<(), String> {
    let hosts_path = default_hosts_path();
    
    let mut content = String::new();
    content.push_str("# Hosts file managed by edHost\n");
    content.push_str("# Generated automatically\n\n");
    
    // 按IP分组hostname
    let mut ip_to_hosts: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    
    for entry in entries {
        if entry.enabled {
            ip_to_hosts
                .entry(entry.ip.clone())
                .or_insert_with(Vec::new)
                .push(entry.hostname.clone());
        }
    }
    
    for (ip, hostnames) in ip_to_hosts {
        // IP和hostname之间用空格分隔
        content.push_str(&format!("{} {}\n", ip, hostnames.join(" ")));
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(&hosts_path)
        .map_err(|e| format!("打开文件错误：{}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入文件错误: {}", e))?;

    Ok(())
}


fn read_file(path: &PathBuf) -> Result<Vec<HostEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = io::BufReader::new(file);

    let mut res = Vec::new();

    let ip_hostname_re = Regex::new(r"^\s*(\d+\.\d+\.\d+\.\d+)\s+([^\s#]+)\s*").unwrap(); // 匹配有效的映射规则
    let comment_re = Regex::new(r"^\s*#\s*(\d+\.\d+\.\d+\.\d+)\s+([^\s#]+)\s*").unwrap(); // 匹配注释行中的映射规则


    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let line = line.trim();

        // 跳过空行和注释
        if line.is_empty() {
            continue;
        }
        if line.starts_with("#") {
            if let Some (caps) = comment_re.captures(&line){
                res.push(HostEntry {
                    ip: caps.get(1).map_or("", |m| m.as_str()).to_string(),
                    hostname: caps.get(2).map_or("", |m| m.as_str()).to_string(),
                    enabled: false
                })
            }
        }

        if let Some (caps) = ip_hostname_re.captures(&line){
            res.push(HostEntry {
                ip: caps.get(1).map_or("", |m| m.as_str()).to_string(),
                hostname: caps.get(2).map_or("", |m| m.as_str()).to_string(),
                enabled: true
            })
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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_hosts, backup_hosts, add_host_entry, save_hosts,open_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
