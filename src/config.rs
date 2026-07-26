use serde::{Deserialize, Serialize};
use std::fs;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GetModuleFileNameW, GetModuleHandleExW,
};

/// Bullet time action mode (Toggle vs Hold).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionMode {
    Toggle,
    Hold,
}

impl Default for ActionMode {
    fn default() -> Self {
        ActionMode::Hold
    }
}

/// Configuration settings for bullet time mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletTimeConfig {
    /// Action mode: "toggle" or "hold"
    #[serde(default)]
    pub action_type: ActionMode,

    /// Target speed multiplier during bullet time (e.g., 0.2)
    #[serde(default = "default_bullet_time_speed")]
    pub bullet_time_speed: f32,

    /// Normal game speed multiplier (default 1.0)
    #[serde(default = "default_normal_speed")]
    pub normal_speed: f32,

    /// Whether to treat Torrent (player mount) the same as the player (normal speed during bullet time)
    #[serde(default = "default_include_torrent")]
    pub include_torrent: bool,

    /// Enable stealth / invisibility effect during bullet time
    #[serde(default = "default_enable_stealth")]
    pub enable_stealth: bool,

    /// SpEffect IDs to apply for stealth during bullet time (e.g. [4100, 4101])
    #[serde(default = "default_stealth_speffect_ids")]
    pub stealth_speffect_ids: Vec<i32>,

    /// Key combinations to activate bullet time (e.g. ["O", "PadRSUp", "lthumbpress+xa"])
    #[serde(default = "default_bullet_time_keys")]
    pub bullet_time_keys: Vec<String>,

    /// Key combinations to deactivate bullet time (e.g. ["P", "PadRSDown", "lthumbpress+xb"])
    #[serde(default = "default_normal_keys")]
    pub normal_keys: Vec<String>,
}

fn default_bullet_time_speed() -> f32 {
    0.2
}

fn default_normal_speed() -> f32 {
    1.0
}

fn default_include_torrent() -> bool {
    true
}

fn default_enable_stealth() -> bool {
    true
}

fn default_stealth_speffect_ids() -> Vec<i32> {
    vec![4100]
}

fn default_bullet_time_keys() -> Vec<String> {
    vec![
        "O".to_string(),
        "lthumbpress+xa".to_string(),
        "PadRSUp".to_string(),
    ]
}

fn default_normal_keys() -> Vec<String> {
    vec![
        "P".to_string(),
        "lthumbpress+xb".to_string(),
        "PadRSDown".to_string(),
    ]
}

impl Default for BulletTimeConfig {
    fn default() -> Self {
        Self {
            action_type: ActionMode::Hold,
            bullet_time_speed: default_bullet_time_speed(),
            normal_speed: default_normal_speed(),
            include_torrent: default_include_torrent(),
            enable_stealth: default_enable_stealth(),
            stealth_speffect_ids: default_stealth_speffect_ids(),
            bullet_time_keys: default_bullet_time_keys(),
            normal_keys: default_normal_keys(),
        }
    }
}

/// Top-level TOML application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub bullet_time: BulletTimeConfig,
}

/// Resolves the absolute directory path where the current DLL is located.
pub fn get_dll_directory() -> Option<PathBuf> {
    let mut hmodule: HMODULE = std::ptr::null_mut();
    unsafe {
        if GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            get_dll_directory as *const u16,
            &mut hmodule,
        ) == 0
        {
            return None;
        }

        let mut buffer = [0u16; 1024];
        let len = GetModuleFileNameW(hmodule, buffer.as_mut_ptr(), buffer.len() as u32);
        if len > 0 {
            let os_str = std::ffi::OsString::from_wide(&buffer[..len as usize]);
            let mut path = PathBuf::from(os_str);
            path.pop(); // Remove DLL filename to get directory
            Some(path)
        } else {
            None
        }
    }
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Loads `er_bullet_time.toml` from the DLL directory, creating a default config if it doesn't exist.
pub fn load_or_create_config() -> AppConfig {
    let mut config_path = get_dll_directory().unwrap_or_else(|| PathBuf::from("."));
    config_path.push("er_bullet_time.toml");

    if !config_path.exists() {
        let default_config = AppConfig::default();
        if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
            let _ = fs::write(&config_path, toml_str);
            tracing::info!("Created default configuration file at: {:?}", config_path);
        }
        return default_config;
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<AppConfig>(&content) {
            Ok(config) => {
                tracing::info!("Successfully loaded configuration from {:?}", config_path);
                config
            }
            Err(err) => {
                tracing::error!(
                    "Failed to parse {:?}: {}. Using default config.",
                    config_path,
                    err
                );
                AppConfig::default()
            }
        },
        Err(err) => {
            tracing::error!(
                "Failed to read {:?}: {}. Using default config.",
                config_path,
                err
            );
            AppConfig::default()
        }
    }
}

/// Returns a reference to the global `AppConfig` singleton.
pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(load_or_create_config)
}
