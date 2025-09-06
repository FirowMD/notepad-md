use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub colorscheme: Option<String>,
    pub recent_files: Option<Vec<String>>,
    pub opened_files: Option<Vec<String>>,
    pub font_size: Option<i32>,
    pub word_wrap: Option<bool>,
    pub show_invisibles: Option<bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            colorscheme: Some("NotepadMD".to_string()),
            recent_files: Some(vec![]),
            opened_files: Some(vec![]),
            font_size: Some(14),
            word_wrap: Some(false),
            show_invisibles: Some(false),
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let config_file = fs::File::open(path).map_err(|e| e.to_string())?;
        serde_json::from_reader(config_file).map_err(|e| e.to_string())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let config_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, config_str).map_err(|e| e.to_string())
    }
}

pub struct AppData {
    pub app_config: AppConfig,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            app_config: AppConfig::new(),
        }
    }
}

pub struct Storage {
    pub app_data: Mutex<AppData>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            app_data: Mutex::new(AppData::new()),
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let config_dir = app_handle
            .path()
            .config_dir()
            .map_err(|e| e.to_string())?;
        Ok(config_dir.join("notepad-md.json"))
    }

    pub fn load_config(app_handle: &tauri::AppHandle) -> Result<(), String> {
        let config_path = Self::get_config_path(app_handle)?;

        let config = if config_path.exists() {
            AppConfig::from_file(&config_path)?
        } else {
            let default_config = AppConfig::default();
            default_config.save_to_file(&config_path)?;
            default_config
        };

        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        app_data.app_config = config;

        Ok(())
    }

    pub fn save_config(app_handle: &tauri::AppHandle, config: AppConfig) -> Result<(), String> {
        let config_path = Self::get_config_path(app_handle)?;
        config.save_to_file(&config_path)?;

        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        app_data.app_config = config;

        Ok(())
    }

    pub fn get_config(app_handle: &tauri::AppHandle) -> Result<AppConfig, String> {
        let storage = app_handle.state::<Storage>();
        let app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        Ok(app_data.app_config.clone())
    }

    pub fn add_to_opened_files(app_handle: &tauri::AppHandle, path: String) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        let mut opened_files = app_data.app_config.opened_files.take().unwrap_or_default();
        
        if !opened_files.contains(&path) {
            opened_files.push(path);
            app_data.app_config.opened_files = Some(opened_files);
            
            let config_to_save = app_data.app_config.clone();
            drop(app_data);
            
            Self::save_config(app_handle, config_to_save)?;
        }
        
        Ok(())
    }

    pub fn add_to_recent_files(app_handle: &tauri::AppHandle, path: String) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        let mut recent_files = app_data.app_config.recent_files.take().unwrap_or_default();
        
        recent_files.retain(|p| p != &path);
        recent_files.insert(0, path);
        
        if recent_files.len() > 100 {
            recent_files.truncate(100);
        }
        
        app_data.app_config.recent_files = Some(recent_files);
        
        let config_to_save = app_data.app_config.clone();
        drop(app_data);
        
        Self::save_config(app_handle, config_to_save)?;
        
        Ok(())
    }

    pub fn remove_from_opened_files(app_handle: &tauri::AppHandle, path: &str) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        if let Some(mut opened_files) = app_data.app_config.opened_files.take() {
            opened_files.retain(|p| p != path);
            app_data.app_config.opened_files = Some(opened_files);
            
            let config_to_save = app_data.app_config.clone();
            drop(app_data);
            
            Self::save_config(app_handle, config_to_save)?;
        }
        
        Ok(())
    }

    pub fn clear_opened_files(app_handle: &tauri::AppHandle) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        app_data.app_config.opened_files = Some(vec![]);
        
        let config_to_save = app_data.app_config.clone();
        drop(app_data);
        
        Self::save_config(app_handle, config_to_save)?;
        
        Ok(())
    }
}

#[tauri::command]
pub fn load_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    ConfigManager::load_config(&app_handle)
}

#[tauri::command]
pub fn save_config(app_handle: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    ConfigManager::save_config(&app_handle, config)
}

#[tauri::command]
pub fn get_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    ConfigManager::get_config(&app_handle)
}