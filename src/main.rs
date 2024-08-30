mod weather;
mod files;

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

use files::get_all_wallpapers;
use weather::get_current_weather;


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
            self.tags.iter()
                .map(|w| format!("{w:?}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Wallpaper {
    //TODO: call when displaying
    fn render_preview(&self) {
        let conf = viuer::Config {
            absolute_offset: false,
            x: 0,
            y: 0,
            width: Some(32),
            height: Some(18),
            ..Default::default()
        };

        let _ = viuer::print_from_file(self.path.to_str().unwrap(), &conf);
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
        write!(f, "{} ({})", self.condition.to_string().bold(), if self.is_day {"daytime"} else {"night-time"})
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
    let curr_weather = get_current_weather();
    let suitable_wallpapers = get_suitable_wallpapers(&curr_weather);
    
    println!("Current Weather: {}", curr_weather);
    println!("Suitable Wallpapers: {:?}", suitable_wallpapers);
}

/* Filter out wallpapers that do not have current weather as tag */
fn get_suitable_wallpapers(weather: &Weather) -> HashSet<Wallpaper> {
    get_all_wallpapers()
        .into_iter()
        .filter(|w| w.tags.contains(&weather.condition))
        .collect()
}
