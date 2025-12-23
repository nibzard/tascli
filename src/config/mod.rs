use std::{
    fs,
    path::PathBuf,
};

use nanoserde::{DeJson, SerJson};

const DB_NAME: &str = "tascli.db";
const DEFAULT_DATA_DIR: &[&str] = &[".local", "share", "tascli"];
const CONFIG_PATH: &[&str] = &[".config", "tascli", "config.json"];

#[derive(Default, DeJson, SerJson)]
pub struct Config {
    /// Only supports full path.
    #[nserde(default)]
    pub data_dir: String,
    /// NLP configuration settings
    #[nserde(default)]
    pub nlp: NLPConfigSection,
}

#[derive(DeJson, SerJson)]
pub struct NLPConfigSection {
    /// Whether NLP is enabled
    #[nserde(default)]
    pub enabled: bool,
    /// OpenAI API key
    #[nserde(default)]
    pub api_key: String,
    /// Model to use (default: gpt-5-nano)
    #[nserde(default)]
    pub model: String,
    /// Whether to fallback to traditional commands on error
    #[nserde(default)]
    pub fallback_to_traditional: bool,
    /// Whether to cache command parses
    #[nserde(default)]
    pub cache_commands: bool,
    /// Context window size for conversation
    #[nserde(default)]
    pub context_window: usize,
    /// Maximum API calls per minute
    #[nserde(default)]
    pub max_api_calls_per_minute: u32,
    /// API base URL (can be overridden for testing)
    #[nserde(default)]
    pub api_base_url: String,
    /// API request timeout in seconds (default: 30)
    #[nserde(default)]
    pub timeout_seconds: u64,
    /// Whether to show preview before executing commands
    #[nserde(default)]
    pub preview_enabled: bool,
    /// Whether to auto-confirm preview without asking
    #[nserde(default)]
    pub auto_confirm: bool,
}

impl Default for NLPConfigSection {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            model: "gpt-5-nano".to_string(),
            fallback_to_traditional: true,
            cache_commands: true,
            context_window: 10,
            max_api_calls_per_minute: 20,
            api_base_url: "https://api.openai.com/v1".to_string(),
            timeout_seconds: 30,
            preview_enabled: true,
            auto_confirm: false,
        }
    }
}

pub fn get_data_path() -> Result<PathBuf, String> {
    let home_dir = home::home_dir().ok_or_else(|| String::from("cannot find home directory"))?;
    let data_dir = match get_config_data_dir(home_dir.clone()) {
        Some(dir_path) => str_to_pathbuf(dir_path)?,
        None => DEFAULT_DATA_DIR.iter().fold(home_dir, |p, d| p.join(d)),
    };
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
    Ok(data_dir.join(DB_NAME))
}

// Quick passthrough for reading config file
// If config file do not exist, return quickly
fn get_config_data_dir(home_dir: PathBuf) -> Option<String> {
    let config_path = CONFIG_PATH.iter().fold(home_dir, |p, d| p.join(d));
    if !config_path.exists() {
        return None;
    }
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => return None,
    };
    let config: Config = match DeJson::deserialize_json(&config_content) {
        Ok(config) => config,
        Err(_) => return None,
    };
    if config.data_dir.is_empty() {
        None
    } else {
        Some(config.data_dir)
    }
}

fn str_to_pathbuf(dir_path: String) -> Result<PathBuf, String> {
    if dir_path.starts_with("~") {
        // We have already executed home_dir previously
        let mut path_buf = home::home_dir().unwrap();
        if dir_path.len() > 2 && dir_path.starts_with("~/") {
            path_buf.push(&dir_path[2..]);
        }
        Ok(path_buf)
    } else if dir_path.starts_with("/") {
        Ok(PathBuf::from(dir_path))
    } else {
        Err(format!("data directory must be absolute or home relative, and start with '~' or '/', it cannot be {}", dir_path))
    }
}

/// Get the full configuration from the config file
pub fn get_config() -> Result<Config, String> {
    let home_dir = home::home_dir().ok_or_else(|| String::from("cannot find home directory"))?;
    let config_path = CONFIG_PATH.iter().fold(home_dir, |p, d| p.join(d));

    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(Config::default());
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: Config = DeJson::deserialize_json(&config_content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    Ok(config)
}

/// Save configuration to the config file
pub fn save_config(config: &Config) -> Result<(), String> {
    let home_dir = home::home_dir().ok_or_else(|| String::from("cannot find home directory"))?;
    let config_path = CONFIG_PATH.iter().fold(home_dir, |p, d| p.join(d));

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_json = config.serialize_json();
    fs::write(&config_path, config_json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Get just the NLP configuration
pub fn get_nlp_config() -> Result<crate::nlp::NLPConfig, String> {
    let config = get_config()?;
    let nlp_section = config.nlp;

    Ok(crate::nlp::NLPConfig {
        enabled: nlp_section.enabled,
        api_key: if nlp_section.api_key.is_empty() { None } else { Some(nlp_section.api_key) },
        model: nlp_section.model,
        fallback_to_traditional: nlp_section.fallback_to_traditional,
        cache_commands: nlp_section.cache_commands,
        context_window: nlp_section.context_window,
        max_api_calls_per_minute: nlp_section.max_api_calls_per_minute,
        api_base_url: nlp_section.api_base_url,
        timeout_seconds: nlp_section.timeout_seconds,
        preview_enabled: nlp_section.preview_enabled,
        auto_confirm: nlp_section.auto_confirm,
    })
}

/// Update NLP configuration
pub fn update_nlp_config(nlp_config: &crate::nlp::NLPConfig) -> Result<(), String> {
    let mut config = get_config()?;

    config.nlp = NLPConfigSection {
        enabled: nlp_config.enabled,
        api_key: nlp_config.api_key.clone().unwrap_or_default(),
        model: nlp_config.model.clone(),
        fallback_to_traditional: nlp_config.fallback_to_traditional,
        cache_commands: nlp_config.cache_commands,
        context_window: nlp_config.context_window,
        max_api_calls_per_minute: nlp_config.max_api_calls_per_minute,
        api_base_url: nlp_config.api_base_url.clone(),
        timeout_seconds: nlp_config.timeout_seconds,
        preview_enabled: nlp_config.preview_enabled,
        auto_confirm: nlp_config.auto_confirm,
    };

    save_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_pathbuf_with_tilde() {
        // Test with just "~"
        let result = str_to_pathbuf("~".to_string()).unwrap();
        let expected = home::home_dir().unwrap();
        assert_eq!(result, expected);

        let result = str_to_pathbuf("~/".to_string()).unwrap();
        let expected = home::home_dir().unwrap();
        assert_eq!(result, expected);

        let result = str_to_pathbuf("~/some/path".to_string()).unwrap();
        let expected = home::home_dir().unwrap().join("some").join("path");
        assert_eq!(result, expected);

        let result = str_to_pathbuf("some/relative/path".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("must be absolute or home relative"));
    }
}
