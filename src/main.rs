mod weather;
mod files;

use std::{fs, path::PathBuf, vec, thread, time, process, io};
use wallpaper;
use console::Term;

const CHECK_INTERVAL_SEC: u64 = 15 * 60; 


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
        process::exit(0);
    }
    
    Term::stdout().clear_screen().unwrap();
    
    loop
    {
        let mut weather_tags: Vec<&str> = vec![];
        weather::set_tags(&mut weather_tags);
        
        let suitable_paths: Vec<PathBuf> = files::get_suitable_wallpapers(&wallpapers, weather_tags);
        
        // for wallpaper in &suitable_wallpapers 
        // {
        //     println!("{}", &wallpaper.file_name().to_string_lossy());
        // }
        
        let ref chosen_wallpaper_path = suitable_paths[files::get_rand_index(&suitable_paths)];
        println!("Chosen: {}", chosen_wallpaper_path.file_name().unwrap().to_str().unwrap());
        
        wallpaper::set_from_path(chosen_wallpaper_path.to_str().unwrap()).unwrap();

        thread::sleep(time::Duration::from_secs(CHECK_INTERVAL_SEC));

        Term::stdout().clear_screen().unwrap();
    }
}

