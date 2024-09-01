mod weather;
mod wallpaper;

use std::{
    thread, 
    time::Duration, 
    collections::HashSet, 
    fmt::{self, Display}, 
    hash::{Hash, Hasher}, 
    path::{Path, PathBuf}, 
};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use dialoguer::Select;
use strum_macros::EnumIter;

use viuer;
use colored::Colorize;

use serde::{Deserialize, Serialize};

use wallpaper::{edit_all_tags, get_all_wallpapers};
use weather::get_current_weather;

const PREVIEW_WIDTH: u32 = 64;
const PREVIEW_OFFSET: u16 = 8;

const WAIT_SECS: u64 = 5 * 60;

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


fn main() {
    loop {
        let choice = Select::new()
            .with_prompt("Weather Wallpaper")
            .item("Start")
            .item("Edit wallpaper tags")
            .item("Settings")
            .item("Quit")
            .default(0)
            .report(false)
            .interact()
            .unwrap();
    
        match choice {
            0 => initiate(),
            1 => edit_all_tags(),
            2 => todo!(), /* TODO: allow changing of refresh times, etc. */
            3 => break, /* Quit */
            _ => unreachable!()
        }
    }
}

/* Start wallpaper setting */
fn initiate() {
    loop {
        update_wallpaper();
        println!("Will update in {}s", WAIT_SECS);
        thread::sleep(Duration::from_secs(WAIT_SECS));
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
