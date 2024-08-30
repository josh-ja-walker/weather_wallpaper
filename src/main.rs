mod weather;
mod wallpaper;

use std::{
    collections::HashSet, 
    path::PathBuf, 
    fmt::{self, Display}, 
    hash::{Hash, Hasher}, 
};

use strum_macros::{Display, EnumIter};

use viuer;    
use colored::Colorize;

use serde::{Deserialize, Serialize};

use wallpaper::{edit_all_tags, get_all_wallpapers};
use weather::get_current_weather;

const PREVIEW_WIDTH: u32 = 64;
const PREVIEW_OFFSET: u16 = 8;

#[derive(Debug, Clone)]
pub struct Wallpaper {
    filename: String,
    path: PathBuf,
    tags: HashSet<WeatherCond>
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
        write!(f, "{} ({:?})\n tags: {}", 
            self.filename.bold(), 
            self.path,
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

    //TODO: call when displaying
    /* Output preview of photo in terminal */
    fn render_preview(&self) {
        let conf = viuer::Config {
            absolute_offset: false,
            x: PREVIEW_OFFSET,
            y: 0,
            width: Some(PREVIEW_WIDTH),
            height: None,
            ..Default::default()
        };
        
        println!(" image: ");
        let _ = viuer::print_from_file(self.path.to_str().unwrap(), &conf);
    }

    /* Print info and image to console */
    fn print(&self) {
        println!("{self}");
        self.render_preview();
    }
    
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Weather {
    condition: WeatherCond,
    is_day: bool
}

impl Display for Weather {
    /* Print weather conditions */
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", 
            self.condition.to_string().bold(), 
            if self.is_day {"daytime"} else {"night-time"})
    }
}

//TODO: add other conditions
#[derive(Display, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
enum WeatherCond {
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
    edit_all_tags();

    let curr_weather = get_current_weather();
    let suitable_wallpapers = get_suitable_wallpapers(&curr_weather);
    
    println!("Current Weather: {}", curr_weather);

    println!("Suitable Wallpapers: ");
    suitable_wallpapers.iter().for_each(|w| w.print());
    
}

/* Filter out wallpapers that do not have current weather as tag */
fn get_suitable_wallpapers(weather: &Weather) -> HashSet<Wallpaper> {
    get_all_wallpapers()
        .into_iter()
        .filter(|w| w.tags.contains(&weather.condition))
        .collect()
}
