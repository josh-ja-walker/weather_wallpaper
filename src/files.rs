use std::path::{PathBuf, Path};
use std::{fs, io, ffi, env};
use regex;
use dirs::picture_dir;

pub fn get_wallpaper_dir() -> PathBuf 
{
    let wallpaper_dir: PathBuf = PathBuf::from(picture_dir().unwrap()).join("Wallpapers (Weather)");

    if !&wallpaper_dir.exists() 
    {
        fs::create_dir(&wallpaper_dir.to_str().unwrap()).unwrap();
    }
    
    return wallpaper_dir;
}

pub fn move_default_wallpapers(to: &Path) 
{
    let to = to.join("Default Wallpapers");
    let default_folder = env::current_exe().unwrap().parent().unwrap().join("Default Wallpapers");

    if !default_folder.exists() || to.exists()
    {
        return;
    }
    
    fs::rename(default_folder, to).unwrap();
}

pub fn get_valid_wallpapers(wallpaper_dir: &PathBuf, get_defaults: bool) -> Vec<fs::DirEntry> 
{
    let files: Result<Vec<fs::DirEntry>, io::Error> = fs::read_dir(&wallpaper_dir).unwrap().collect();
    let mut files = files.unwrap();

    if get_defaults 
    {
        let defaults = fs::read_dir(&wallpaper_dir.join("Default Wallpapers"));

        if defaults.is_ok()  
        {
            let defaults = defaults.unwrap().collect();
            match defaults 
            {
                Ok(mut defaults) => files.append(&mut defaults),
                Err(_) => (),
            }
        }
    }
    
    println!("Valid Files:");

    let mut i = 0;
    while i < files.len()
    {
        let ref file: fs::DirEntry = files[i];
    
        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap();
        
        println!("\t{}", file_name);
        
        let ext_valid = check_extension(file.path());
        if ext_valid
        {
            println!("removed {0}. extension is {1}valid", file_name, if ext_valid {"in"} else {""});
            files.remove(i);
        }
        else
        {
            i += 1;
        }
    }
    
    return files;
}

fn check_extension(file_path: PathBuf) -> bool 
{
    let supported_files_re: regex::Regex = regex::Regex::new(r"(png|jpg|bmp)").unwrap();
    return supported_files_re.is_match(file_path.extension().unwrap_or(ffi::OsStr::new("")).to_str().unwrap());
}

pub fn rename_files(wallpaper_dir: &PathBuf) 
{
    let files = get_valid_wallpapers(wallpaper_dir, false);
    
    for i in 0..files.len()
    {
        let mut tag_nums: Vec<i32> = vec![];
        let file_name = files[i].file_name();
        let file_name = file_name.to_str().expect("file name cannot be converted to string");
        
        println!("File: {}", file_name);
        
        let mut file_name = String::from(i.to_string());
        let mut skip = false;

        println!("0-Any;  ");
        println!("1-Sunny;  ");
        println!("2-Rainy;  ");
        println!("3-Cloudy;  ");
        println!("4-Partly Cloudy;  ");
        println!("5-Hot;  ");
        println!("6-Cold;  ");
        println!("7-Windy;  ");
        println!("8-Foggy;  ");
        println!("9-Night;  ");
        println!("10-Clear;  ");
        println!("11-Next file;  ");
        println!("12-Skip renaming;");
        println!("Input tags: ");
        
        loop
        {
            let inp = crate::get_input("");

            if inp == ""
            {
                if tag_nums.is_empty() { skip = true; }
                break;
            }

            let inp = inp.parse::<i32>();
            
            let inp = match inp {
                Ok(inp) => inp,
                Err(_) => break,                
            };
            
            if tag_nums.contains(&inp)
            {
                continue;
            } 

            match inp 
            {
                0 => {tag_nums.clear(); break},
                1..=10 => tag_nums.push(inp),
                12 => return,
                _ => {skip = true; break}
            }
        }
        if skip { continue; }
        
        tag_nums.sort();
        for num in tag_nums.clone() 
        {
            let tag = match num 
            {
                1 => "sun",
                2 => "rain",
                3 => "cloud",
                4 => "part_cl",
                5 => "hot",
                6 => "cold",
                7 => "wind",
                8 => "fog",
                9 => "night",
                10 => "clear",
                _ => "",
            };

            file_name = file_name + &format!("-{}", tag);
        }
        
        file_name = format!("{}.{}", file_name, files[i].path().extension().unwrap().to_str().unwrap()); 

        if file_name != files[i].file_name().to_str().unwrap() 
        {
            let to = get_wallpaper_dir()
                .join(&file_name);
            
            fs::rename(files[i].path(), to).unwrap();
            println!("renamed {}\n", file_name);
        }

    }
}

pub fn get_suitable_wallpapers(valid_files: &Vec<fs::DirEntry>, weather_tags: Vec<&str>) -> Vec<PathBuf> 
{
    let re: regex::RegexSet = regex::RegexSet::new(&weather_tags).unwrap();

    let mut suitable_paths = vec![];
    let mut any_paths = vec![];

    let night_re: regex::Regex = regex::Regex::new(r"night").unwrap();
    
    let mut max_tags: usize = 0;
    
    // println!("", file_name);
    for file in valid_files
    {
        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap();
        
        println!("{}", file_name);

        let num_matches = re.matches(&file_name).iter().count();

        if night_re.is_match(&file_name) == weather_tags.contains(&"night") 
        {
            if num_matches >= max_tags
            {
                if num_matches > max_tags 
                {
                    max_tags = num_matches;
                    
                    suitable_paths.clear();
                }
    
                suitable_paths.push(file.path());
            }     
            else if num_matches == 0 
            {
                let file_name_num = file_name[0..file_name.len() - 4].parse::<i128>();
                
                match file_name_num {
                    Ok(_) => any_paths.push(file.path()),
                    Err(_error) => (),            
                };
            }
        }
    }
    
    suitable_paths.append(&mut any_paths);
    
    // println!("Suitable files: ");
    // for path in suitable_paths.clone() 
    // {
    //     println!("\t- {}", path.file_name().unwrap().to_str().unwrap());
    // }

    return suitable_paths;
}

pub fn get_rand_index <T> (suitable_wallpapers: &Vec<T>) -> usize 
{
    let len: usize = suitable_wallpapers.len();
    return fastrand::usize(0..len);
}
