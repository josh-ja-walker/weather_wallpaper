mod weather;
mod files;

use std::{fs, path::PathBuf, vec, thread, time, io};

// use wallpaper;
// use more_wallpapers;
use console::Term;
use winconsole::window;

const SHOW_TIME: u64 = 2; 

fn main() {
    let update_interval: u64 = (60.0 * get_input("Enter time between updating (in minutes): ").parse::<f64>().unwrap_or(15.0)) as u64;
    if update_interval == (15 * 60) || update_interval == 0 { println!("Set update time to 15 min"); }
    
    let hide_window: bool = get_input("Hide the window? Y/N: ").to_ascii_lowercase() == "y";    

    let mut popup: bool = false;
    if hide_window 
    {
        popup = get_input("Show window when refreshed? Y/N: ").to_ascii_lowercase() == "y";    
    }

    let wallpaper_dir: PathBuf = files::get_wallpaper_dir();

    files::move_default_wallpapers(&wallpaper_dir);

    let rename_inp = get_input("\nRename files?: Y/N");
    if rename_inp.to_ascii_lowercase() == "y" {
        files::rename_files(&wallpaper_dir);
    }
    
    get_input("");
    Term::stdout().clear_screen().unwrap();
    
    let mut wallpapers: Vec<fs::DirEntry> = files::get_valid_wallpapers(&wallpaper_dir, true);
    
    while wallpapers.len() <= 0 {
        get_input(&format!("Add compatible wallpapers (png, jpg or bmp) to {}", wallpaper_dir.to_str().unwrap()));
        wallpapers = files::get_valid_wallpapers(&wallpaper_dir, true);
    }
    
    Term::stdout().clear_screen().unwrap();
    
    loop {
        if popup {
            window::show(popup);
        }
        
        let mut weather_tags: Vec<&str> = vec![];
        weather::set_tags(&mut weather_tags);
        
        let suitable_paths: Vec<PathBuf> = files::get_suitable_wallpapers(&wallpapers, weather_tags);
        
        // let ref chosen_wallpaper_path = suitable_paths[files::get_rand_index(&suitable_paths)];
        // println!("Chosen: {}", chosen_wallpaper_path.file_name().unwrap().to_str().unwrap());
        
        // wallpaper::set_from_path(chosen_wallpaper_path.to_str().unwrap()).unwrap();
        more_wallpapers::set_random_wallpapers_from_vec(suitable_paths, more_wallpapers::Mode::Center).unwrap();

        thread::sleep(time::Duration::from_secs(SHOW_TIME));

        if hide_window 
        {
            window::hide();
        }

        thread::sleep(time::Duration::from_secs(update_interval));
        
        Term::stdout().clear_screen().unwrap();
    }
}

pub fn get_input(msg: &str) -> String {
    println!("{}", msg);
    let mut inp = String::new();            
    io::stdin().read_line(&mut inp).unwrap();
    inp = inp.trim().to_string();
    return inp;
}

pub fn title(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}