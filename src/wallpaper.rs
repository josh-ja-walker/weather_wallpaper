use std::{fs, io};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use dialoguer::Input;
use dialoguer::MultiSelect;
use dialoguer::Select;

use regex::Regex;

use dirs::picture_dir;

use strum::IntoEnumIterator;

use crate::{Wallpaper, Weather, WeatherTag, PREVIEW_WIDTH};

const VALID_EXTS: [&'static str; 3] = ["png", "jpg", "bmp"];
const WALLPAPER_TAGS_FILE: &str = "wallpaper_tags.json";


/* Retrieve all saved wallpapers */
pub fn get_all_wallpapers() -> HashSet<Wallpaper> {
    let files: fs::ReadDir = fs::read_dir(wallpaper_dir_path())
        .expect("Could not read wallpaper directory");

    let tag_map: HashMap<String, Weather> = load_wallpaper_weather()
        .expect("Could not load tag map");

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


/* Load wallpaper from directory */
fn load_wallpaper(file: fs::DirEntry, weather_map: &HashMap<String, Weather>) -> Wallpaper {
    let filename = file.file_name().into_string().unwrap();

    Wallpaper {
        filename: filename.clone(),
        path: file.path(),
        weather: weather_map
            .get(&filename)
            .unwrap_or(&Weather::default())
            .clone(),
    }
}


/* Edit the tags of all wallpapers */
pub fn edit_all_tags() {
    let mut wallpapers = get_all_wallpapers()
        .into_iter()
        .collect::<Vec<Wallpaper>>();

    edit_menu(0, &mut wallpapers);

    save_wallpaper_weather(&wallpapers.into_iter().collect()).unwrap();
}

/* Edit the tags of a wallpaper */
fn edit_menu(index: usize, wallpapers: &mut Vec<Wallpaper>) {
    if index >= wallpapers.len() {
        return;
    }

    print!("{}. ", index);

    match edit_tags(&mut wallpapers[index]) {
        Ok(_) => edit_menu(index + 1, wallpapers),
        Err(e) if e.kind() == io::ErrorKind::Interrupted => interrupted_menu(index, wallpapers),
        error => error.unwrap(), 
    }
}

/* Edit the tags of a wallpaper */
fn edit_tags(wallpaper: &mut Wallpaper) -> io::Result<()> {
    wallpaper.print(PREVIEW_WIDTH);

    let tag_options: Vec<(WeatherTag, String, bool)> = WeatherTag::iter()
        .map(|cond| (cond.clone(), cond.to_string(), wallpaper.weather.tags.contains(&cond)))
        .collect();

    let options: Vec<(String, bool)> = tag_options
        .iter()
        .map(|(_, s, b)| (s.clone(), b.clone()))
        .collect();

    let interrupt_error = io::Error::new(io::ErrorKind::Interrupted, "Control character [esc, q] pressed");
    let input = MultiSelect::new()
        .with_prompt("Select weather tags")
        .items_checked(&options)
        .report(false)
        .interact_opt()
        .unwrap()
        .ok_or(interrupt_error)?;
    
    let interrupt_error = io::Error::new(io::ErrorKind::Interrupted, "Control character [esc, q] pressed");
    let day_night = Select::new()
        .with_prompt("Select day or night")
        .item("Day")
        .item("Night")
        .default(!wallpaper.weather.is_day as usize)
        .report(false)
        .interact_opt()
        .unwrap()
        .ok_or(interrupt_error)?;

    /* Update tags */
    wallpaper.weather.tags = input.into_iter()
        .map(|i| tag_options[i].0.clone())
        .collect();

    /* Update day/night */
    wallpaper.weather.is_day = day_night == 0;

    Ok(())
}

/* Interrupted editing of tags (skip/goto/quit) */
fn interrupted_menu(index: usize, wallpapers: &mut Vec<Wallpaper>) {
    let control = Select::new()
        .with_prompt("Interrupted!")
        .item("Next")
        .item("Prev")
        .item("Go to ")
        .item("Reset all tags") //TODO
        .item("Quit")
        .default(0)
        .report(false)
        .interact()
        .unwrap();

    let new_index = match control {
        /* Next */
        0 => index + 1,

        /* Prev */
        1 => index.checked_sub(1).unwrap_or(0),

        /* Goto x */ 
        2 => Input::new()
            .with_prompt("Enter index of wallpaper to edit")
            .validate_with(|input: &String| 
                match input.parse::<usize>() {
                    Ok(x) if x < wallpapers.len() => Ok(()),
                    Ok(_) => Err("out of range"),
                    Err(_) => Err("not a number"),
                })
            .interact()
            .unwrap()
            .parse::<usize>()
            .unwrap(),

        /* Quit */
        3 => wallpapers.len(),

        _ => unreachable!(),
    };

    edit_menu(new_index, wallpapers)
}


/* Save map of tags associated with each file */
fn save_wallpaper_weather(wallpapers: &HashSet<Wallpaper>) -> io::Result<()> {
    let weather_map: HashMap<String, Weather> = HashMap::from_iter(wallpapers
        .into_iter()
        .cloned()
        .map(|Wallpaper { filename, weather, .. }| 
            (filename, weather)
        )
    );

    fs::write(saved_weather_path(), serde_json::to_string_pretty(&weather_map)?)
}

/* Load map of tags associated with each file */
fn load_wallpaper_weather() -> io::Result<HashMap<String, Weather>> {
    let contents = fs::read_to_string(saved_weather_path())
        .unwrap_or(String::from("{}"));

    let parsed: HashMap<String, Weather> = serde_json::from_str(&contents)
        .expect("Could not parse saved weather file");

    Ok(parsed)
}

/* Helper function to get path to file of saved tags */
fn saved_weather_path() -> PathBuf {
    wallpaper_dir_path().join(WALLPAPER_TAGS_FILE)
}
