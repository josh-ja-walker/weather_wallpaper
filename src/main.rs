mod weather;
mod wallpaper;

use std::{
    io,
    fs, 
    thread, 
    time::Duration,
    collections::HashSet, 
    fmt::{self, Display}, 
    hash::{Hash, Hasher}, 
    path::{Path, PathBuf}, 
};

use console::Term;
use dirs::picture_dir;
use indicatif::{ProgressBar, ProgressStyle};
use rand::prelude::*;
use rand::distributions::WeightedIndex;

use dialoguer::{Input, Select};
use strum_macros::EnumIter;

use viuer;
use colored::Colorize;

use serde::{Deserialize, Serialize};

use wallpaper::{edit_all_tags, get_all_wallpapers};
use weather::get_current_weather;

const PREVIEW_WIDTH: u32 = 64;
const PREVIEW_OFFSET: u16 = 8;

const INTERVAL_MILLIS: u64 = 5 * 60 * 1000;

const SAVED_SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone)]
pub struct Wallpaper {
    filename: String,
    path: PathBuf,
    weather: Weather
}

impl Eq for Wallpaper {}
impl PartialEq for Wallpaper {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Hash for Wallpaper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl AsRef<Path> for Wallpaper {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl Display for Wallpaper {

    /* Print name, path and tags of Wallpaper */
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})\n weather depicted: {}", 
            self.filename.bold(), 
            self.path.to_str().unwrap().to_string().dimmed(),
            self.weather
        )
    }
}

impl Wallpaper {

    /* Output preview of photo in terminal */
    fn render_preview(&self, width: u32) {
        let conf = viuer::Config {
            absolute_offset: false,
            x: PREVIEW_OFFSET,
            y: 0,
            width: Some(width),
            height: None,
            ..Default::default()
        };
        
        let _ = viuer::print_from_file(self.path.to_str().unwrap(), &conf);
    }

    /* Print info and image to console */
    fn print(&self, width: u32) {
        println!("{self}");
        println!(" image: ");

        self.render_preview(width);
        println!();
    }
    
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Weather {
    tags: HashSet<WeatherTag>,
    is_day: bool
}

/* Default weather */
impl Default for Weather {
    fn default() -> Self {
        Self { 
            tags: HashSet::new(), 
            is_day: true 
        }
    }
}

/* Print weather conditions */
impl Display for Weather {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", 
            if self.tags.is_empty() {
                String::from("none").dimmed()
            } else {
                self.tags.iter()
                    .map(WeatherTag::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
                    .bold()
            },

            if self.is_day {"daytime"} else {"night-time"}
        )
    }

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
enum WeatherTag {
    Clear,
    Sun,
    Cloud,
    PartCloud,
    Fog,
    Rain,
    Storm,
    Snow
}

/* Settings config */
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    interval: u64, /* Refresh interval in millis */
    hide_window: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            interval: INTERVAL_MILLIS, 
            hide_window: false, 
        }
    }
}

impl Config {
    fn interval_mins(&self) -> f32 {
        self.interval as f32 / (60.0 * 1000.0)
    }
}


fn main() {
    let mut config = load_settings().unwrap_or_default();

    loop {
        let choice = Select::new()
            .with_prompt("Weather Wallpaper")
            .item("Start")
            .item("Tags")
            .item("Settings")
            .item("Quit")
            .default(0)
            .report(false)
            .interact()
            .unwrap();
    
        match choice {
            0 => set_wallpaper(&config),
            1 => edit_all_tags(),
            2 => edit_settings(&mut config), /* TODO: allow changing of refresh times, etc. */
            3 => break, /* Quit */
            _ => unreachable!()
        }
    }
}

/* Get wallpaper directory (nested in Picture directory) */
fn wallpaper_dir_path() -> PathBuf {
    let wallpaper_dir: PathBuf = PathBuf::from(picture_dir().expect("No picture directory found"))
        .join("weather_wallpapers");

    /* Create wallpaper directory if it doesn't exist */
    if !&wallpaper_dir.exists() {
        fs::create_dir(wallpaper_dir.clone())
            .expect("Could not create wallpaper directory");
    }
    
    return wallpaper_dir;
}


/* Start wallpaper setting */
fn set_wallpaper(config: &Config) {
    loop {
        update_wallpaper();
    
        let bar_style = ProgressStyle::with_template("{msg}\n[{elapsed_precise}] {wide_bar:.white/gray} ({eta})\t\t")
            .unwrap();
        
        let pb = ProgressBar::new(config.interval)
            .with_style(bar_style)
            .with_message("Time remaining until refresh:");
        
        for _ in 0..pb.length().unwrap() {
            thread::sleep(Duration::from_millis(1));
            pb.inc(1);
        }
        
        pb.finish_and_clear();
        println!("Now refreshing...");

        thread::sleep(Duration::from_secs(1));
        Term::stdout().clear_screen().unwrap();
    }
}

/* Set the wallpaper */
fn update_wallpaper() {
    let curr_weather = get_current_weather();
    println!("Current Weather: {}", curr_weather);

    let chosen = choose_wallpaper(curr_weather, get_all_wallpapers());

    print!("Chosen: ");
    chosen.print(PREVIEW_WIDTH);

    wallpaper_setting::set_from_path(chosen.path.to_str().unwrap()).unwrap();
}

/* Choose random wallpaper */
fn choose_wallpaper(weather: Weather, wallpapers: HashSet<Wallpaper>) -> Wallpaper {
    let mut rng = thread_rng();
    
    let mut day_filtered: Vec<&Wallpaper> = wallpapers
        .iter()
        .filter(|w| w.weather.is_day == weather.is_day)
        .collect();

    if day_filtered.is_empty() {
        day_filtered = wallpapers.iter().collect();
    }
    
    let tag_weighted: Vec<(usize, &&Wallpaper)> = day_filtered
        .iter()
        .map(|w| (w.weather.tags.intersection(&weather.tags).count(), w)) 
        .filter(|(num_match, _)| *num_match > 0)
        .collect();

    if tag_weighted.is_empty() {
        day_filtered.into_iter().choose(&mut rng).unwrap()
    } else {
        let dist = WeightedIndex::new(tag_weighted.iter().map(|item| item.0)).unwrap();
        tag_weighted[dist.sample(&mut rng)].1
    }.clone()
}


/* Set refresh time, window shows, etc. */
fn edit_settings(config: &mut Config) {
    let choice = Select::new()
        .with_prompt("Edit settings")
        .item(format!("Set refresh interval [{} mins]", config.interval_mins()))
        .item(format!("Toggle window hide behavior [{}]", "TODO"))
        .item("Restore default settings")
        .item("Back")
        .default(0)
        .report(false)
        .interact_opt()
        .unwrap();

    if choice.is_none() { return; }

    match choice.unwrap() {
        0 => set_interval(config),
        1 => todo!(),
        2 => todo!(),
        3 => (),
        _ => unreachable!()
    };

    save_settings(config).expect("Could not save settings");
}

/* Save settings to .json file */
fn save_settings(config: &Config) -> io::Result<()> {
    fs::write(saved_settings_path(), serde_json::to_string_pretty(config)?)
}

/* Load settings from .json file */
fn load_settings() -> io::Result<Config> {
    let contents = fs::read_to_string(saved_settings_path())?;
    let config = serde_json::from_str(&contents)?;
    Ok(config)
}

/* Helper function to get path to file of saved settings */
fn saved_settings_path() -> PathBuf {
    wallpaper_dir_path().join(SAVED_SETTINGS_FILE)
}

/* Handle input for refresh interval */
fn set_interval(config: &mut Config) {
    let mins = Input::<f32>::new()
        .with_prompt(format!("Set refresh interval [{} mins]", config.interval_mins())) 
        .validate_with(|x: &f32| 
            if *x > 0.0 { Ok(()) } else { Err("Cannot be non-positive") } )
        .interact()
        .unwrap();

    config.interval = (mins * 60.0 * 1000.0) as u64;
}
