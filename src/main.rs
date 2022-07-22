mod weather;
mod files;

use std::{fs, path::PathBuf, vec, thread, time, process, io};

use wallpaper;
use console::Term;
use winconsole::window;

const CHANGE_INTERVAL: u64 = 15 * 60; 
const SHOW_TIME: u64 = 10; 


fn main() 
{
    let wallpaper_dir: PathBuf = files::get_wallpaper_dir();
    
    println!("Rename files?: Y/N");
    let mut inp = String::new();            
    io::stdin().read_line(&mut inp).unwrap();
    inp = inp.trim().to_string();

    if inp.to_ascii_lowercase() == "y"
    {
        files::rename_files(&wallpaper_dir);
    }
    
    Term::stdout().clear_screen().unwrap();
    
    let wallpapers: Vec<fs::DirEntry> = files::get_valid_wallpapers(&wallpaper_dir);
    
    if wallpapers.len() <= 0 {
        println!("no compatible images in {}", wallpaper_dir.to_str().unwrap());
        process::exit(1);
    }
    
    Term::stdout().clear_screen().unwrap();
    
    loop
    {
        window::show(true);
        
        let mut weather_tags: Vec<&str> = vec![];
        weather::set_tags(&mut weather_tags);
        
        let suitable_paths: Vec<PathBuf> = files::get_suitable_wallpapers(&wallpapers, weather_tags);
        
        let ref chosen_wallpaper_path = suitable_paths[files::get_rand_index(&suitable_paths)];
        println!("Chosen: {}", chosen_wallpaper_path.file_name().unwrap().to_str().unwrap());
        
        wallpaper::set_from_path(chosen_wallpaper_path.to_str().unwrap()).unwrap();
        
        thread::sleep(time::Duration::from_secs(SHOW_TIME));
        window::hide();
        thread::sleep(time::Duration::from_secs(CHANGE_INTERVAL));
        
        Term::stdout().clear_screen().unwrap();
    }
}

