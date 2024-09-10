use std::{fs, io};
use std::collections::HashSet;
use std::path::PathBuf;

use dirs::picture_dir;
use regex::Regex;

use crate::{wallpaper, Wallpaper};

const VALID_EXTS: [&'static str; 3] = ["png", "jpg", "bmp"];


/* Retrieve all wallpapers */
pub fn load_all_wallpapers() -> HashSet<Wallpaper> {
    let files = fs::read_dir(wallpapers_path().unwrap())
        .expect("Could not read wallpaper directory")
        .filter(|file| is_valid(file)) /* Remove invalid files */
        .map(|file| Wallpaper::new(file.unwrap())); /* Map to a wallpaper */

    let saved = wallpaper::load_wallpapers()
        .unwrap_or(HashSet::new())
        .into_iter()
        .filter(|wallpaper| wallpaper.is_valid());

    let wallpapers: HashSet<Wallpaper> = saved
        .chain(files)
        .collect();

    wallpaper::save_wallpapers(&wallpapers).unwrap();

    wallpapers
}

/* Check the file is valid */
fn is_valid(file: &io::Result<fs::DirEntry>) -> bool {
    match file {
        Ok(file) => check_extension(file.path()),
        Err(_) => false,
    }
}

/* Check the file's extension is valid */
fn check_extension(file_path: PathBuf) -> bool {
    let valid_exts: Regex = Regex::new(&format!("({})", VALID_EXTS.join("|"))).unwrap();
    
    let file_ext = file_path.extension()
        .map_or("", |ext| ext.to_str().unwrap());
        
    valid_exts.is_match(&file_ext)
}

/* Get wallpaper directory (nested in Picture directory) */
pub fn wallpapers_path() -> io::Result<PathBuf> {
    let wallpaper_dir: PathBuf = PathBuf::from(
        picture_dir()
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "Picture directory not found"))?
        ).join("weather_wallpapers");

    /* Create wallpaper directory if it doesn't exist */
    if !&wallpaper_dir.exists() {
        fs::create_dir(&wallpaper_dir)?;
    }

    Ok(wallpaper_dir)
}

/* Get data directory (nested in Wallpapers directory) */
pub fn data_path() -> io::Result<PathBuf> {
    let settings_dir: PathBuf = wallpapers_path()?.join("data");

    /* Create wallpaper directory if it doesn't exist */
    if !&settings_dir.exists() {
        fs::create_dir(&settings_dir)?;
    }

    Ok(settings_dir)
}

