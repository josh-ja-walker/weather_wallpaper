use std::{fs, io, path::PathBuf};

use console::Term;
use dialoguer::{Input, Select};
use serde::{Deserialize, Serialize};

use crate::{files, format_items};

const INTERVAL_MILLIS: u64 = 5 * 60 * 1000;

const SAVED_SETTINGS_FILE: &str = "settings.json";

/* Settings config */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    interval: u64, /* Refresh interval in millis */
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            interval: INTERVAL_MILLIS, 
        }
    }
}

impl Config {
    pub fn interval_mins(&self) -> f32 {
        self.interval as f32 / (60.0 * 1000.0)
    }

    pub fn interval_millis(&self) -> u64 {
        self.interval
    }
}


/* Set refresh time, window shows, etc. */
pub fn edit_settings(config: &mut Config) {
    let choice = Select::new()
        .with_prompt("Edit settings")
        .items(&format_items(vec![
            &format!("Set refresh interval [{} mins]", config.interval_mins()),
            "Restore default settings",
            "Back",
        ]))
        .default(0)
        .report(false)
        .interact_opt()
        .unwrap();

    if choice.is_none() { return; }

    match choice.unwrap() {
        0 => set_interval(config),
        1 => *config = Config::default(),
        2 => return,
        _ => unreachable!()
    };

    save_settings(config).expect("Could not save settings");
}

/* Handle input for refresh interval */
fn set_interval(config: &mut Config) {
    let prompt = format!("Set refresh interval [{} mins]", config.interval_mins());

    /* Ensure input is positive */
    let validator = |x: &f32| -> Result<(), &str> {
        if *x > 0.0 { 
            Ok(()) 
        } else { 
            Err("Cannot be non-positive") 
        }
    };

    /* Get interval input in minutes */
    let mins = Input::<f32>::new()
        .with_prompt(prompt)
        .validate_with(validator)
        .interact()
        .unwrap();
            
    /* Update interval (in millis) */
    config.interval = (mins * 60.0 * 1000.0) as u64;
    
    /* Clear input */
    Term::stdout().clear_last_lines(1).unwrap();
}


/* Save settings to .json file */
fn save_settings(config: &Config) -> io::Result<()> {
    fs::write(saved_settings_path(), serde_json::to_string_pretty(config)?)
}

/* Load settings from .json file */
pub fn load_settings() -> io::Result<Config> {
    let contents = fs::read_to_string(saved_settings_path())?;
    let config = serde_json::from_str(&contents)?;
    Ok(config)
}

/* Helper function to get path to file of saved settings */
fn saved_settings_path() -> PathBuf {
    files::wallpapers_path().join(SAVED_SETTINGS_FILE)
}
