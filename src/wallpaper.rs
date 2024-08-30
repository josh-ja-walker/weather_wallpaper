use std::fs;
use std::io;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use regex::Regex;
use dirs::picture_dir;
use strum::IntoEnumIterator;

use crate::{Wallpaper, WeatherCond};

const VALID_EXTS: [&'static str; 3] = ["png", "jpg", "bmp"];
const SAVED_TAGS_FILE: &str = "./tags.json";


/* Retrieve all saved wallpapers */
pub fn get_all_wallpapers() -> HashSet<Wallpaper> {
    let files: fs::ReadDir = fs::read_dir(get_wallpaper_dir())
        .expect("Could not read wallpaper directory");

    let tag_map: HashMap<String, HashSet<WeatherCond>> = load_tag_map().expect("Could not load tag map");

    files.map(|file| file.unwrap())
        .filter(|file| is_valid(file)) /* Remove invalid files */
        .map(|file| load_wallpaper(file, &tag_map)) /* Map to Wallpaper */
        .collect::<HashSet<Wallpaper>>() /* Collect into vector */
}

/* Check the file is valid */
fn is_valid(file: &fs::DirEntry) -> bool {
    check_extension(file.path())
}

/* Check the file's extension is valid */
fn check_extension(file_path: PathBuf) -> bool {
    let valid_exts: Regex = Regex::new(
        &format!("({})", VALID_EXTS.join("|"))).unwrap();
    
    let file_ext = file_path.extension()
        .map_or("", |ext| ext.to_str().unwrap());
        
    valid_exts.is_match(&file_ext)
}

/* Get wallpaper directory (nested in Picture directory) */
fn get_wallpaper_dir() -> PathBuf {
    let wallpaper_dir: PathBuf = PathBuf::from(picture_dir().expect("No picture directory found"))
        .join("weather_wallpapers");

    /* Create wallpaper directory if it doesn't exist */
    if !&wallpaper_dir.exists() {
        fs::create_dir(wallpaper_dir.clone())
            .expect("Could not create wallpaper directory");
    }
    
    return wallpaper_dir;
}


/* Load wallpaper from directory */
fn load_wallpaper(file: fs::DirEntry, tag_map: &HashMap<String, HashSet<WeatherCond>>) -> Wallpaper {
    let filename = file.file_name().into_string().unwrap();

    Wallpaper {
        filename: filename.clone(),
        path: file.path(),
        tags: tag_map
            .get(&filename)
            .unwrap_or(&HashSet::new())
            .clone(),
    }
}


/* Edit the tags of all wallpapers */
pub fn edit_all_tags() {
    let wallpapers = get_all_wallpapers();

    let updated_wallpapers: HashSet<Wallpaper> = wallpapers
        .into_iter()
        .map(|w| edit_tags(w).unwrap())
        .collect();

    save_tag_map(&updated_wallpapers);
}

/* Edit the tags of a wallpaper */
fn edit_tags(wallpaper: Wallpaper) -> Result<Wallpaper, Box<dyn std::error::Error>> {
    wallpaper.print();

    print!("\n");

    println!("Choose tags:");
    WeatherCond::iter()
    .map(|cond| cond.synonyms())
    .map(|syns| syns.join(", ").to_lowercase())
    .enumerate()
    .for_each(|(index, syns)| println!(" {index}: {syns}"));

    // println!(" s: skip this wallpaper");
    // println!(" g <index>: Go to nth wallpaper");
    // println!(" q: quit editing tags");

    println!("----------------");

    let mut buffer = String::new();
    let stdin = io::stdin(); 
    stdin.read_line(&mut buffer)?;

    /* TODO: Handle goto or skip or quit options */
    
    /* Parse tag from numerical input */
    let tag: WeatherCond = WeatherCond::iter()
        .nth(buffer.trim().parse::<usize>()?)
        .ok_or("Invalid choice of tag")?;

    let mut new_wallpaper = wallpaper.clone();
    new_wallpaper.tags.insert(tag);

    Ok(new_wallpaper)
}


//TODO: save tags and allow editing of maps
/* Save map of tags associated with each file */
fn save_tag_map(wallpapers: &HashSet<Wallpaper>) -> io::Result<()> {
    let tag_map: HashMap<String, Vec<WeatherCond>> = HashMap::from_iter(wallpapers
        .into_iter()
        .cloned()
        .map(|Wallpaper { filename, tags, .. }| 
            (filename, tags.into_iter().collect::<Vec<WeatherCond>>())
        )
    );

    fs::write(saved_tags_path(), serde_json::to_string_pretty(&tag_map)?)
}

/* Load map of tags associated with each file */
fn load_tag_map() -> io::Result<HashMap<String, HashSet<WeatherCond>>> {
    let contents = fs::read_to_string(saved_tags_path())
        .unwrap_or(String::from("{}"));

    let parsed: HashMap<String, Vec<WeatherCond>> = serde_json::from_str(&contents)
        .expect("Could not parse tags file");

    Ok(parsed.into_iter()
        .map(|(filename, value)| (filename, HashSet::from_iter(value)))
        .collect())
}

/* Helper function to get path to file of saved tags */
fn saved_tags_path() -> PathBuf {
    picture_dir()
        .unwrap()
        .join(SAVED_TAGS_FILE)
}
