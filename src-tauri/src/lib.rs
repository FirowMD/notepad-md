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
    let mut app_config: AppConfig = serde_json::from_reader(config_file).map_err(|e| e.to_string())?;
    let app_data = app_handle.state::<Storage>();
    let mut app_data = app_data.app_data.lock().map_err(|e| e.to_string())?;
    
    if let (Some(existing_files), Some(new_files)) = (&app_data.app_config.opened_files, &app_config.opened_files) {
        let mut merged_files: Vec<String> = existing_files.clone();
        for file in new_files {
            if !merged_files.contains(file) {
                merged_files.insert(0, file.clone());
            }
        }
        app_config.opened_files = Some(merged_files);
    }
    
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

    if let (Some(existing_files), Some(new_files)) = (&app_data.app_config.opened_files, &default_config.opened_files) {
        let mut merged_files: Vec<String> = existing_files.clone();
        for file in new_files {
            if !merged_files.contains(file) {
                merged_files.insert(0, file.clone());
            }
        }
        let mut updated_config = default_config;
        updated_config.opened_files = Some(merged_files);
        app_data.app_config = updated_config;
    } else {
        app_data.app_config = default_config;
    }

    Ok(())
}

#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().collect();
    let mut files_to_open: Vec<String> = Vec::new();

    // Process CLI arguments (skip the first arg as it's the program path)
    for arg in cli_args.iter().skip(1) {
        if let Ok(canonical_path) = std::fs::canonicalize(arg) {
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
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Focus the main window when attempting to launch another instance
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_focus();
        }));
    }

    builder
        .manage(Mutex::new(WatcherState::new()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Storage {
            app_data: Mutex::new(AppData {
                app_config: AppConfig {
                    colorscheme: None,
                    recent_files: None,
                    opened_files: Some(files_to_open),
                    font_size: None,
                    word_wrap: None,
                    show_invisibles: None
                },
            }),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            load_config,
            save_config,
            read_file,
            run_explorer,
            save_file,
            rename_file,
            watch_file,
            unwatch_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
