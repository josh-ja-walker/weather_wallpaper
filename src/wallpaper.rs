use std::{fs, io};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use dialoguer::Input;
use dialoguer::MultiSelect;
use dialoguer::Select;

use regex::Regex;

use dirs::picture_dir;

use strum::IntoEnumIterator;

use crate::{Wallpaper, WeatherTag, PREVIEW_WIDTH};

const VALID_EXTS: [&'static str; 3] = ["png", "jpg", "bmp"];
const SAVED_TAGS_FILE: &str = "./tags.json";


/* Retrieve all saved wallpapers */
pub fn get_all_wallpapers() -> HashSet<Wallpaper> {
    let files: fs::ReadDir = fs::read_dir(get_wallpaper_dir())
        .expect("Could not read wallpaper directory");

    let tag_map: HashMap<String, HashSet<WeatherTag>> = load_tag_map()
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
fn load_wallpaper(file: fs::DirEntry, tag_map: &HashMap<String, HashSet<WeatherTag>>) -> Wallpaper {
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
    let mut wallpapers = get_all_wallpapers()
        .into_iter()
        .collect::<Vec<Wallpaper>>();

    edit_menu(0, &mut wallpapers);

    save_tag_map(&wallpapers.into_iter().collect()).unwrap();
}

/* Edit the tags of a wallpaper */
fn edit_menu(index: usize, wallpapers: &mut Vec<Wallpaper>) {
    if index >= wallpapers.len() {
        return;
    }

    print!("{}. ", index);

    match edit_tags(&mut wallpapers[index]) {
        Ok(_) => edit_menu(index + 1, wallpapers),
        Err(_) => control_edit_menu(index, wallpapers),
    }
}

/* Edit the tags of a wallpaper */
fn edit_tags(wallpaper: &mut Wallpaper) -> io::Result<()> {
    wallpaper.print(PREVIEW_WIDTH);

    let cond_items: Vec<(WeatherTag, String, bool)> = WeatherTag::iter()
        .map(|cond| (
            cond.clone(), cond.to_string(), wallpaper.tags.contains(&cond)
        ))
        .collect();

    let items: Vec<(String, bool)> = cond_items
        .iter()
        .map(|(_, s, b)| (s.clone(), b.clone()))
        .collect();

    let input = MultiSelect::new()
        .with_prompt("Select weather tags")
        .items_checked(&items)
        .report(false)
        .interact_opt()
        .unwrap()
        .ok_or(io::Error::other("Control character [esc, q] pressed to exit menu"))?;
    
    /* Update tags */
    wallpaper.tags = input.into_iter()
        .map(|i| cond_items[i].0.clone())
        .collect();

    Ok(())
}

/* Control editing of tags (skip/goto/quit) */
fn control_edit_menu(index: usize, wallpapers: &mut Vec<Wallpaper>) {
    let control = Select::new()
        .with_prompt("Setting tags interrupted")
        .item("Next")
        .item("Prev")
        .item("Go to ")
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
fn save_tag_map(wallpapers: &HashSet<Wallpaper>) -> io::Result<()> {
    let tag_map: HashMap<String, Vec<WeatherTag>> = HashMap::from_iter(wallpapers
        .into_iter()
        .cloned()
        .map(|Wallpaper { filename, tags, .. }| 
            (filename, tags.into_iter().collect::<Vec<WeatherTag>>())
        )
    );

    fs::write(saved_tags_path(), serde_json::to_string_pretty(&tag_map)?)
}

/* Load map of tags associated with each file */
fn load_tag_map() -> io::Result<HashMap<String, HashSet<WeatherTag>>> {
    let contents = fs::read_to_string(saved_tags_path())
        .unwrap_or(String::from("{}"));

    let parsed: HashMap<String, Vec<WeatherTag>> = serde_json::from_str(&contents)
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
