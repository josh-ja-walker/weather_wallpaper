use std::{cmp, collections::HashSet, fmt::{self, Display}, fs, hash::{Hash, Hasher}, io, path::{Path, PathBuf}};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{Error, files, weather::Weather};

const PREVIEW_WIDTH: u32 = 64;
const PREVIEW_OFFSET: u16 = 8;

const FAV_WEIGHT_MULT: usize = 2;

const WALLPAPER_TAGS_FILE: &str = "wallpaper_tags.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    filename: String,
    path: PathBuf,
    pub weather: Weather,
    favourited: bool
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

impl PartialOrd for Wallpaper {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.filename.partial_cmp(&other.filename)
    }
}

impl Ord for Wallpaper {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.filename.cmp(&other.filename)
    }
}

impl AsRef<Path> for Wallpaper {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

/* Print name, path and tags of Wallpaper */
impl Display for Wallpaper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{} ({})\n Weather depicted: {}", 
            self.favourited.then_some("\u{2605} ").unwrap_or(""),
            self.filename.bold(), 
            self.path.display().to_string().dimmed(),
            self.weather
        )
    }
}

impl Wallpaper {
    
    /* Load wallpaper from directory */
    pub fn new(file: fs::DirEntry) -> Wallpaper {
        let filename = file.file_name().into_string().unwrap();

        Wallpaper {
            filename: filename.clone(),
            path: file.path(),
            weather: Weather::default(),
            favourited: false,
        }
    }

    pub fn is_favourited(&self) -> bool {
        self.favourited
    }

    pub fn toggle_favourited(&mut self) {
        self.favourited = !self.favourited;
    }

    pub fn is_valid(&self) -> bool {
        self.path.exists()
    }

    pub fn get_weight(&self, weather: &Weather) -> usize {
        let matching_tags = self.weather.tags().intersection(weather.tags()).count();

        if self.favourited {
            matching_tags * FAV_WEIGHT_MULT
        } else {
            matching_tags
        }
    }

    /* Print info and image to console */
    pub fn print(&self) {
        println!("{self}");

        println!(" Image: ");
        self.render_preview().unwrap();
        
        println!();
    }

    /* Output preview of photo in terminal */
    fn render_preview(&self) -> Result<(u32, u32), Error> {
        let conf = viuer::Config {
            absolute_offset: false,
            
            x: PREVIEW_OFFSET,
            y: 0,
            
            width: Some(PREVIEW_WIDTH),
            height: None,
            
            truecolor: true,

            ..Default::default()
        };
        
        viuer::print_from_file(self.path.to_str().unwrap(), &conf)
            .map_err(|_| Error::ImagePrintFail)
    }

    pub fn set(&self) -> Result<(), Error> {
        let path_str = self.path.to_str().unwrap();

        wallpaper_setting::set_from_path(path_str)
            .map_err(|_| Error::InvalidWallpaper)
    }

}


/* Save map of tags associated with each file */
pub fn save_wallpapers(wallpapers: &HashSet<Wallpaper>) -> io::Result<()> {
    fs::write(wallpaper_tags_path()?, serde_json::to_string_pretty(&wallpapers)?)
}

/* Load map of tags associated with each file */
pub fn load_wallpapers() -> io::Result<HashSet<Wallpaper>> {
    let contents = fs::read_to_string(wallpaper_tags_path()?)?;
    let parsed: Vec<Wallpaper> = serde_json::from_str(&contents)?;

    Ok(parsed.into_iter().collect::<HashSet<Wallpaper>>())
}

/* Helper function to get path to file of saved tags */
fn wallpaper_tags_path() -> io::Result<PathBuf> {
    files::data_path().and_then(|path| Ok(path.join(WALLPAPER_TAGS_FILE)))
}
