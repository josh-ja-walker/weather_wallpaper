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
const SMALL_PREVIEW_WIDTH: u32 = 32;
const PREVIEW_OFFSET: u16 = 8;

const WAIT_SECS: u64 = 5 * 60;

#[derive(Debug, Clone)]
pub struct Wallpaper {
    filename: String,
    path: PathBuf,
    tags: HashSet<WeatherTag>
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
        write!(f, "{} ({})\n tags: {}", 
            
            self.filename.bold(), 

            self.path.to_str().unwrap().to_string().dimmed(),
            
            if self.tags.is_empty() {
                String::from("none")
            } else {
                self.tags.iter()
                    .map(|tag| tag.synonyms()[0].to_lowercase().clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            }
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

impl Display for Weather {
    /* Print weather conditions */
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", 
            self.tags.iter()
                .map(WeatherTag::to_string)
                .collect::<Vec<String>>()
                .join(", ")
                .bold(), 

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
            0 => start(),
            1 => edit_all_tags(),
            2 => todo!(), /* TODO: allow changing of refresh times, etc. */
            3 => break, /* Quit */
            _ => unreachable!()
        }
    }
}

/* Start wallpaper setting */
fn start() {
    loop {
        set_wallpaper();
        thread::sleep(Duration::from_secs(WAIT_SECS));
    }
}

/* Set the wallpaper */
fn set_wallpaper() {
    let curr_weather = get_current_weather();
    let suitable_wallpapers = get_suitable_wallpapers(&curr_weather);
    
    println!("Current Weather: {}", curr_weather);

    // println!("Suitable Wallpapers: ");
    // suitable_wallpapers.iter().for_each(|w| w.print(SMALL_PREVIEW_WIDTH));

    let chosen = choose_wallpaper(curr_weather, suitable_wallpapers);
    print!("Chosen: ");
    chosen.print(PREVIEW_WIDTH);

    wallpaper_setting::set_from_path(chosen.path.as_os_str().to_str().unwrap()).unwrap();
}

/* Choose random wallpaper */
fn choose_wallpaper(weather: Weather, suitable: HashSet<Wallpaper>) -> Wallpaper {
    let mut rng = thread_rng();

    let items: Vec<(usize, Wallpaper)> = suitable
        .into_iter()
        .map(|w| (w.tags.intersection(&weather.tags).count(), w)) 
        .collect();

    let dist = WeightedIndex::new(items.iter().map(|item| item.0)).unwrap();

    items[dist.sample(&mut rng)].1.clone()
}

/* Filter wallpapers with no matching tags */
fn get_suitable_wallpapers(weather: &Weather) -> HashSet<Wallpaper> {
    get_all_wallpapers()
        .into_iter()
        // .filter(|w| weather.is_day && ) TODO
        .filter(|w| w.tags.intersection(&weather.tags).next().is_some())
        .collect()
}
    