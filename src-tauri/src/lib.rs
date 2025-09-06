// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use notify::{Watcher, RecursiveMode, Event};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Manager;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    colorscheme: Option<String>,
    recent_files: Option<Vec<String>>,
    opened_files: Option<Vec<String>>,
    font_size: Option<i32>,
    word_wrap: Option<bool>,
    show_invisibles: Option<bool>
}

struct AppData {
    app_config: AppConfig,
}

struct Storage {
    app_data: Mutex<AppData>,
}

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
fn load_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    let config_dir = app_handle.path().config_dir().unwrap();
    let config_path = config_dir.to_str().unwrap();
    let config_file_path = format!("{}/notepad-md.json", config_path);

    if Path::new(&config_file_path).exists() {
        read_config(&config_file_path, &app_handle)?;
    } else {
        create_default_config(&config_file_path, &app_handle)?;
    }

    Ok(())
}

#[tauri::command]
fn save_config(app_handle: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let config_dir = app_handle.path().config_dir().unwrap();
    let config_path = config_dir.to_str().unwrap();
    let config_file_path = format!("{}/notepad-md.json", config_path);

    let config_str = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    std::fs::write(&config_file_path, config_str).map_err(|e| e.to_string())?;

    Ok(()) 
}

fn read_config(config_file_path: &str, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let config_file = std::fs::File::open(config_file_path).map_err(|e| e.to_string())?;
    let app_config: AppConfig = serde_json::from_reader(config_file).map_err(|e| e.to_string())?;
    let app_data = app_handle.state::<Storage>();
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    
    app_data.app_config = app_config;
    Ok(())
}

fn create_default_config(config_file_path: &str, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let _ = std::fs::File::create(config_file_path).map_err(|e| e.to_string())?;
    
    let default_config = AppConfig {
        colorscheme: Some("NotepadMD".to_string()),
        recent_files: Some(vec![]),
        opened_files: Some(vec![]),
        font_size: Some(14),
        word_wrap: Some(false),
        show_invisibles: Some(false)
    };

    let default_config_str = serde_json::to_string(&default_config).map_err(|e| e.to_string())?;
    std::fs::write(config_file_path, default_config_str).map_err(|e| e.to_string())?;

    let app_data = app_handle.state::<Storage>();
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;

    app_data.app_config = default_config;

    Ok(())
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
fn get_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    let app_data = app_handle.state::<Storage>();
    let app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    Ok(AppConfig {
        colorscheme: app_data.app_config.colorscheme.clone(),
        recent_files: app_data.app_config.recent_files.clone(),
        opened_files: app_data.app_config.opened_files.clone(),
        font_size: app_data.app_config.font_size.clone(),
        word_wrap: app_data.app_config.word_wrap.clone(),
        show_invisibles: app_data.app_config.show_invisibles.clone()
    })
}

fn add_to_opened_files(app_handle: &tauri::AppHandle, path: String) -> Result<(), String> {
    let app_data = app_handle.state::<Storage>();
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    
    let mut opened_files = app_data.app_config.opened_files.take().unwrap_or_default();
    
    if !opened_files.contains(&path) {
        opened_files.push(path);
        app_data.app_config.opened_files = Some(opened_files);
        
        save_config(app_handle.clone(), AppConfig {
            colorscheme: app_data.app_config.colorscheme.clone(),
            recent_files: app_data.app_config.recent_files.clone(),
            opened_files: app_data.app_config.opened_files.clone(),
            font_size: app_data.app_config.font_size.clone(),
            word_wrap: app_data.app_config.word_wrap.clone(),
            show_invisibles: app_data.app_config.show_invisibles.clone()
        })?;
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().collect();
    let mut files_to_open: Vec<String> = Vec::new();
    let mut skip_single_instance = false;

    for arg in cli_args.iter().skip(1) {
        if arg == "--no-single-instance" {
            skip_single_instance = true;
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
                let _ = load_config(app.clone());
                
                for path in argv.iter().skip(1) {
                    if path != "--no-single-instance" {
                        if let Ok(canonical_path) = std::fs::canonicalize(path) {
                            if canonical_path.exists() {
                                if let Some(path_str) = canonical_path.to_str() {
                                    add_to_opened_files(&app, path_str.to_string()).unwrap();
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
        .manage(Storage {
            app_data: Mutex::new(AppData {
                app_config: AppConfig {
                    colorscheme: None,
                    recent_files: None,
                    opened_files: None,
                    font_size: None,
                    word_wrap: None,
                    show_invisibles: None
                },
            }),
        })
        .setup(move |app| {
            let _ = load_config(app.handle().clone());
            
            for file_path in files_to_open {
                let _ = add_to_opened_files(&app.handle(), file_path);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            load_config,
            save_config,
            read_file,
            calculate_file_hash_command,
            run_explorer,
            open_in_new_window,
            save_file,
            rename_file,
            watch_file,
            unwatch_file
        ]);
    
    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
