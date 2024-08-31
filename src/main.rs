mod weather;
mod wallpaper;

use std::{
    collections::HashSet, 
    path::PathBuf, 
    fmt::{self, Display}, 
    hash::{Hash, Hasher}, 
};

use dialoguer::Select;
use strum_macros::{Display, EnumIter};

use viuer;    
use colored::Colorize;

use serde::{Deserialize, Serialize};

use wallpaper::{edit_all_wallpaper_tags, get_all_wallpapers};
use weather::get_current_weather;

const PREVIEW_WIDTH: u32 = 64;
const PREVIEW_OFFSET: u16 = 8;

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
                    .map(|w| format!("{w:?}"))
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

#[derive(Display, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
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
            .default(0)
            .report(false)
            .interact()
            .unwrap();
    
        match choice {
            0 => set_wallpaper(),
            1 => edit_all_wallpaper_tags(),
            2 => todo!(), /* TODO: allow changing of refresh times, etc. */
            _ => unreachable!()
        }
    }
}

fn set_wallpaper() {
    let curr_weather = get_current_weather();
    let suitable_wallpapers = get_suitable_wallpapers(&curr_weather);
    
    println!("Current Weather: {}", curr_weather);
    
    println!("Suitable Wallpapers: ");
    suitable_wallpapers.iter().for_each(|w| w.print(32));
}

/* Filter out wallpapers that do not have current weather as tag */
fn get_suitable_wallpapers(weather: &Weather) -> HashSet<Wallpaper> {
    get_all_wallpapers()
        .into_iter()
        /* Filter out wallpapers with NO matching tags */
        /* TODO: rank wallpapers with more matching tags as more preferable */
        /* TODO: allow any tag selected with none */
        .filter(|w| w.tags.intersection(&weather.tags).next().is_some()) 
        .collect()
}
