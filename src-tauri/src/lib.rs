// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use notify::{Watcher, RecursiveMode, Event};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

mod config;
use config::{Storage, ConfigManager};
use chrono::Local;

struct WatcherState {
    watchers: HashMap<String, notify::RecommendedWatcher>,
}

impl WatcherState {
    fn new() -> Self {
        Self {
            watchers: HashMap::new(),
        }
    }
}

fn calculate_file_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Serialize, Deserialize)]
struct FileData {
    content: String,
    hash: String,
}


#[tauri::command]
fn read_file(path: &str, encoding: Option<String>) -> Result<FileData, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();
    
    if file_size > 100 * 1024 * 1024 {
        return Err("File too large (>100MB). Large files are not supported.".to_string());
    }
    
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    
    let content = if let Some(enc) = encoding {
        match enc.to_uppercase().as_str() {
            "UTF-8" | "UTF8" => String::from_utf8_lossy(&bytes).into_owned(),
            "UTF-16LE" => {
                let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&bytes);
                decoded.into_owned()
            },
            "UTF-16BE" => {
                let (decoded, _, _) = encoding_rs::UTF_16BE.decode(&bytes);
                decoded.into_owned()
            },
            "WINDOWS-1252" => {
                let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
                decoded.into_owned()
            },
            _ => String::from_utf8_lossy(&bytes).into_owned(),
        }
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };

    let hash = calculate_file_hash(&content);
    
    Ok(FileData { content, hash })
}

#[tauri::command]
fn calculate_file_hash_command(content: &str) -> String {
    calculate_file_hash(content)
}

#[tauri::command]
fn save_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_file(old_path: String, new_path: String) -> Result<(), String> {
    std::fs::rename(old_path, new_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn run_explorer(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(_) = Command::new("nautilus").args(["--select", path]).spawn() {
            return Ok(());
        }
        if let Ok(_) = Command::new("dolphin").args(["--select", path]).spawn() {
            return Ok(());
        }
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn open_in_new_window(path: &str) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable: {}", e))?;
    
    #[cfg(target_os = "windows")]
    {
        Command::new(current_exe)
            .args(["--no-single-instance", path])
            .spawn()
            .map_err(|e| format!("Failed to open new window: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new(current_exe)
            .args(["--no-single-instance", path])
            .spawn()
            .map_err(|e| format!("Failed to open new window: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new(current_exe)
            .args(["--no-single-instance", path])
            .spawn()
            .map_err(|e| format!("Failed to open new window: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
fn watch_file(path: String, window: tauri::Window) -> Result<(), String> {
    let app_handle = window.app_handle();
    let state = app_handle.state::<Mutex<WatcherState>>();
    let mut state = state.lock().unwrap();

    if state.watchers.contains_key(&path) {
        return Ok(());
    }

    let window_clone = window.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    if let Some(path) = event.paths.first() {
                        let _ = window_clone.emit("file-changed", path.to_str().unwrap_or(""));
                    }
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }).map_err(|e| e.to_string())?;

    watcher.watch(&PathBuf::from(&path), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    state.watchers.insert(path, watcher);
    Ok(())
}

#[tauri::command]
fn unwatch_file(path: String, window: tauri::Window) -> Result<(), String> {
    let app_handle = window.app_handle();
    let state = app_handle.state::<Mutex<WatcherState>>();
    let mut state = state.lock().unwrap();
    
    state.watchers.remove(&path);
    Ok(())
}

#[tauri::command]
fn get_monaco_themes(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let notepad_dir = config::ConfigManager::get_notepad_md_dir(&app_handle)?;
    let themes_dir = notepad_dir.join("monaco-editor");
    
    // Create the monaco-editor directory if it doesn't exist
    if !themes_dir.exists() {
        fs::create_dir_all(&themes_dir).map_err(|e| e.to_string())?;
    }
    
    let mut themes = vec!["vs".to_string(), "vs-dark".to_string(), "hc-black".to_string()];
    
    if themes_dir.exists() {
        let entries = fs::read_dir(&themes_dir).map_err(|e| e.to_string())?;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(filename) = path.file_stem() {
                        if let Some(theme_name) = filename.to_str() {
                            themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    Ok(themes)
}

#[tauri::command]
fn read_monaco_theme(app_handle: tauri::AppHandle, theme_name: String) -> Result<String, String> {
    if theme_name == "vs" || theme_name == "vs-dark" || theme_name == "hc-black" {
        return Ok(String::new());
    }
    
    let notepad_dir = config::ConfigManager::get_notepad_md_dir(&app_handle)?;
    let theme_path = notepad_dir.join("monaco-editor").join(format!("{}.json", theme_name));
    
    if !theme_path.exists() {
        return Err(format!("Theme file not found: {}", theme_name));
    }
    
    fs::read_to_string(theme_path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().collect();
    let mut files_to_open: Vec<String> = Vec::new();
    let mut skip_single_instance = false;
    let mut instance_id = String::from("main");

    for arg in cli_args.iter().skip(1) {
        if arg == "--no-single-instance" {
            skip_single_instance = true;
            let timestamp = Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
            instance_id = format!("instance_{}", timestamp);
        } else if let Ok(canonical_path) = std::fs::canonicalize(arg) {
            if canonical_path.exists() {
                if let Some(path_str) = canonical_path.to_str() {
                    files_to_open.push(path_str.to_string());
                }
            }
        }
    }

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        if !skip_single_instance {
            builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
                let _ = ConfigManager::load_config(&app);
                
                for path in argv.iter().skip(1) {
                    if path != "--no-single-instance" {
                        if let Ok(canonical_path) = std::fs::canonicalize(path) {
                            if canonical_path.exists() {
                                if let Some(path_str) = canonical_path.to_str() {
                                    ConfigManager::add_to_opened_files(&app, path_str.to_string()).unwrap();
                                }
                            }
                        }
                    }
                }
                let window = app.get_webview_window("main").unwrap();
                let _ = window.set_focus();
                let _ = window.emit("files-updated", ());
            }));
        }
    }

    let app = builder
        .manage(Mutex::new(WatcherState::new()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Storage::with_instance_id(instance_id.clone()))
        .setup(move |app| {
            let _ = ConfigManager::set_instance_id(&app.handle(), instance_id);
            let _ = ConfigManager::load_config(&app.handle());
            
            for file_path in files_to_open {
                let _ = ConfigManager::add_to_opened_files(&app.handle(), file_path);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::load_config,
            config::save_config,
            read_file,
            calculate_file_hash_command,
            run_explorer,
            open_in_new_window,
            save_file,
            rename_file,
            watch_file,
            unwatch_file,
            get_monaco_themes,
            read_monaco_theme
        ]);
    
    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
