use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalConfig {
    pub colorscheme: Option<String>,
    pub monaco_editor_theme: Option<String>,
    pub font_size: Option<i32>,
    pub word_wrap: Option<bool>,
    pub show_invisibles: Option<bool>,
    pub transparent_mode: Option<bool>,
    pub window_opacity: Option<f32>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            colorscheme: Some("FirowNotepad".to_string()),
            monaco_editor_theme: Some("vs-dark".to_string()),
            font_size: Some(14),
            word_wrap: Some(false),
            show_invisibles: Some(false),
            transparent_mode: Some(false),
            window_opacity: Some(0.85),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstanceConfig {
    pub recent_files: Option<Vec<String>>,
    pub opened_files: Option<Vec<String>>,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            recent_files: Some(vec![]),
            opened_files: Some(vec![]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub colorscheme: Option<String>,
    pub monaco_editor_theme: Option<String>,
    pub recent_files: Option<Vec<String>>, 
    pub opened_files: Option<Vec<String>>, 
    pub font_size: Option<i32>,
    pub word_wrap: Option<bool>,
    pub show_invisibles: Option<bool>,
    pub transparent_mode: Option<bool>,
    pub window_opacity: Option<f32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            colorscheme: Some("FirowNotepad".to_string()),
            monaco_editor_theme: Some("vs-dark".to_string()),
            recent_files: Some(vec![]),
            opened_files: Some(vec![]),
            font_size: Some(14),
            word_wrap: Some(false),
            show_invisibles: Some(false),
            transparent_mode: Some(false),
            window_opacity: Some(0.85),
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_global_and_instance(global: GlobalConfig, instance: InstanceConfig) -> Self {
        Self {
            colorscheme: global.colorscheme,
            monaco_editor_theme: global.monaco_editor_theme,
            font_size: global.font_size,
            word_wrap: global.word_wrap,
            show_invisibles: global.show_invisibles,
            transparent_mode: global.transparent_mode,
            window_opacity: global.window_opacity,
            recent_files: instance.recent_files,
            opened_files: instance.opened_files,
        }
    }

    pub fn to_global(&self) -> GlobalConfig {
        GlobalConfig {
            colorscheme: self.colorscheme.clone(),
            monaco_editor_theme: self.monaco_editor_theme.clone(),
            font_size: self.font_size,
            word_wrap: self.word_wrap,
            show_invisibles: self.show_invisibles,
            transparent_mode: self.transparent_mode,
            window_opacity: self.window_opacity,
        }
    }

    pub fn to_instance(&self) -> InstanceConfig {
        InstanceConfig {
            recent_files: self.recent_files.clone(),
            opened_files: self.opened_files.clone(),
        }
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

impl GlobalConfig {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let config_file = fs::File::open(path).map_err(|e| e.to_string())?;
        serde_json::from_reader(config_file).map_err(|e| e.to_string())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let config_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, config_str).map_err(|e| e.to_string())
    }
}

impl InstanceConfig {
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
    pub instance_id: Option<String>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            app_config: AppConfig::new(),
            instance_id: None,
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
    
    pub fn with_instance_id(instance_id: String) -> Self {
        let mut app_data = AppData::new();
        app_data.instance_id = Some(instance_id);
        Self {
            app_data: Mutex::new(app_data),
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_notepad_md_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let config_dir = app_handle
            .path()
            .config_dir()
            .map_err(|e| e.to_string())?;
        let notepad_dir = config_dir.join("FirowNotepad");
        
        if !notepad_dir.exists() {
            fs::create_dir_all(&notepad_dir).map_err(|e| e.to_string())?;
        }
        
        Ok(notepad_dir)
    }
    
    pub fn get_global_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let notepad_dir = Self::get_notepad_md_dir(app_handle)?;
        Ok(notepad_dir.join("firow-notepad-global.json"))
    }

    pub fn get_instance_config_path(app_handle: &tauri::AppHandle, instance_id: &str) -> Result<PathBuf, String> {
        let notepad_dir = Self::get_notepad_md_dir(app_handle)?;
        let instances_dir = notepad_dir.join("firow-notepad-instances");
        
        if !instances_dir.exists() {
            fs::create_dir_all(&instances_dir).map_err(|e| e.to_string())?;
        }
        
        Ok(instances_dir.join(format!("{}.json", instance_id)))
    }

    pub fn get_legacy_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let config_dir = app_handle
            .path()
            .config_dir()
            .map_err(|e| e.to_string())?;
        Ok(config_dir.join("firow-notepad.json"))
    }

    pub fn load_config(app_handle: &tauri::AppHandle) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        let instance_id = app_data.instance_id.clone().unwrap_or_else(|| "main".to_string());
        
        let global_path = Self::get_global_config_path(app_handle)?;
        let global_config = if global_path.exists() {
            GlobalConfig::from_file(&global_path)?
        } else {
            let config = GlobalConfig::default();
            config.save_to_file(&global_path)?;
            config
        };
        
        let instance_path = Self::get_instance_config_path(app_handle, &instance_id)?;
        let instance_config = if instance_path.exists() {
            InstanceConfig::from_file(&instance_path)?
        } else {
            let config = InstanceConfig::default();
            config.save_to_file(&instance_path)?;
            config
        };
        
        app_data.app_config = AppConfig::from_global_and_instance(global_config, instance_config);
        app_data.instance_id = Some(instance_id);

        Ok(())
    }

    pub fn save_config(app_handle: &tauri::AppHandle, config: AppConfig) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        let instance_id = app_data.instance_id.clone().unwrap_or_else(|| "main".to_string());
        
        let global_config = config.to_global();
        let global_path = Self::get_global_config_path(app_handle)?;
        global_config.save_to_file(&global_path)?;
        
        let instance_config = config.to_instance();
        let instance_path = Self::get_instance_config_path(app_handle, &instance_id)?;
        instance_config.save_to_file(&instance_path)?;
        
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
            
            let instance_id = app_data.instance_id.clone().unwrap_or_else(|| "main".to_string());
            let instance_config = app_data.app_config.to_instance();
            drop(app_data);
            
            let instance_path = Self::get_instance_config_path(app_handle, &instance_id)?;
            instance_config.save_to_file(&instance_path)?;
            
            let storage = app_handle.state::<Storage>();
            let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
            app_data.app_config.opened_files = instance_config.opened_files;
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
        
        let instance_id = app_data.instance_id.clone().unwrap_or_else(|| "main".to_string());
        let instance_config = app_data.app_config.to_instance();
        drop(app_data);
        
        let instance_path = Self::get_instance_config_path(app_handle, &instance_id)?;
        instance_config.save_to_file(&instance_path)?;
        
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        app_data.app_config.recent_files = instance_config.recent_files;
        
        Ok(())
    }

    pub fn remove_from_opened_files(app_handle: &tauri::AppHandle, path: &str) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        if let Some(mut opened_files) = app_data.app_config.opened_files.take() {
            opened_files.retain(|p| p != path);
            app_data.app_config.opened_files = Some(opened_files);
            
            let instance_id = app_data.instance_id.clone().unwrap_or_else(|| "main".to_string());
            let instance_config = app_data.app_config.to_instance();
            drop(app_data);
            
            let instance_path = Self::get_instance_config_path(app_handle, &instance_id)?;
            instance_config.save_to_file(&instance_path)?;
            
            let storage = app_handle.state::<Storage>();
            let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
            app_data.app_config.opened_files = instance_config.opened_files;
        }
        
        Ok(())
    }

    pub fn clear_opened_files(app_handle: &tauri::AppHandle) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        
        app_data.app_config.opened_files = Some(vec![]);
        
        let instance_id = app_data.instance_id.clone().unwrap_or_else(|| "main".to_string());
        let instance_config = app_data.app_config.to_instance();
        drop(app_data);
        
        let instance_path = Self::get_instance_config_path(app_handle, &instance_id)?;
        instance_config.save_to_file(&instance_path)?;
        
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        app_data.app_config.opened_files = instance_config.opened_files;
        
        Ok(())
    }
    
    pub fn set_instance_id(app_handle: &tauri::AppHandle, instance_id: String) -> Result<(), String> {
        let storage = app_handle.state::<Storage>();
        let mut app_data = storage.app_data.lock().map_err(|e| e.to_string())?;
        app_data.instance_id = Some(instance_id);
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
