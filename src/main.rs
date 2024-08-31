mod weather;
mod wallpaper;

use std::{
    thread, 
    time::Duration,
    cmp::Ordering, 
    collections::HashSet, 
    fmt::{self, Display}, 
    hash::{Hash, Hasher}, 
    path::{Path, PathBuf}, 
};

use dialoguer::Select;
use more_wallpapers::{set_random_wallpapers_from_vec, Mode};
use strum_macros::{Display, EnumIter};

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

    println!("Suitable Wallpapers: ");
    suitable_wallpapers.iter().for_each(|w| w.print(SMALL_PREVIEW_WIDTH));

    set_random_wallpapers_from_vec(suitable_wallpapers.into_iter().collect(), Mode::Center)
        .unwrap();
}

/* Select most applicable wallpapers (most matching tags) */
fn get_suitable_wallpapers(weather: &Weather) -> HashSet<Wallpaper> {
    get_all_wallpapers()
        .into_iter()
        .map(|w| (w.tags.intersection(&weather.tags).count(), w)) 
        .fold((0, Vec::new()), |mut acc, (ntags2, w)| {
            match acc.0.cmp(&ntags2) {
                Ordering::Greater => acc,
                Ordering::Equal => {acc.1.push(w); acc},
                Ordering::Less => (ntags2, vec![w]),
            }
        }).1
        .into_iter()
        .collect()
    }
    