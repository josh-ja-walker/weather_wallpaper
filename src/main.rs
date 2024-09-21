use std::{
    thread, time::Duration,
    collections::HashSet, 
    fmt::Display, 
};

use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use dialoguer::Select;
use colored::Colorize;

use rand::{distributions::{WeightedError, WeightedIndex}, prelude::*};

use strum_macros::Display;

mod weather;
mod weather_api;

mod files;
mod wallpaper;
mod wallpaper_tags;

mod settings;

use settings::Config;
use wallpaper::Wallpaper;
use weather::Weather;


#[derive(Display, Debug)]
pub enum Error {
    WeatherFetchFail,
    InvalidWallpaper,
    ImagePrintFail,
    Interrupted,
    InvalidInput,
}

impl From<reqwest::Error> for Error {
    fn from(_err: reqwest::Error) -> Self {
        Error::WeatherFetchFail 
    }
}


fn main() {
    let mut config = settings::load_settings().unwrap_or_default();

    if files::load_all_wallpapers().is_empty() {
        println!("Weather Wallpaper:");
        println!("No wallpapers found. Add wallpapers to {}", 
            files::wallpapers_path().unwrap().display().to_string().bold());
        Term::stdout().read_line().unwrap();
        return;
    }

    loop {
        let choice = Select::new()
            .with_prompt("Weather Wallpaper")
            .items(&format_items(vec![
                "Start", 
                "Tags", 
                "Settings", 
                "Quit"
            ]))
            .default(0)
            .report(false)
            .interact()
            .unwrap();

        match choice {
            0 => start(&config),
            1 => {
                wallpaper_tags::edit_wallpaper_tags();
                Term::stdout().clear_screen().unwrap()
            },
            2 => settings::edit_settings(&mut config).unwrap(),
            3 => break, /* Quit */
            _ => unreachable!()
        }
    }
}

/* Format select options */
fn format_items<T>(options: Vec<T>) -> Vec<String> where T: Display {
    options.iter()
        .map(|option| format!("\u{2022} {option}"))
        .collect()
}

/* Start wallpaper setting */
fn start(config: &Config) {
    let wallpapers: HashSet<Wallpaper> = files::load_all_wallpapers();
    
    loop {
        Term::stdout().clear_screen().unwrap();
        println!("{}", "Weather Wallpaper:".bold());

        let fetch_weather: Result<Weather, Error> = weather::get_current_weather();

        let chosen: &Wallpaper = match fetch_weather {
            Ok(curr_weather) => {
                println!("Current Weather: {}", curr_weather);
                choose_wallpaper(curr_weather, &wallpapers)
            },
            Err(Error::WeatherFetchFail) => {
                println!("No weather found; choosing random wallpaper");
                rand_choice(&wallpapers.iter().collect::<HashSet<&Wallpaper>>())
            },
            _ => unreachable!("API fetch returned unrecoverable error")
        };
    
        print!("Chosen: ");
        chosen.print();
        chosen.set().unwrap();

        render_progress_bar(config);

        println!("Now refreshing...");
        thread::sleep(Duration::from_secs(1));
    }
}

/* Render a progress bar to show how long left until wallpaper refreshes */
fn render_progress_bar(config: &Config) {
    let bar_style = ProgressStyle::with_template("{msg}\n[{elapsed_precise}] {wide_bar} ({eta})\t\t")
        .unwrap();

    let pb = ProgressBar::new(config.interval_millis())
        .with_style(bar_style)
        .with_message("Time remaining until refresh:");

    let step_size = 30;
    for _ in 0..config.interval_millis() / step_size {
        thread::sleep(Duration::from_millis(step_size));
        pb.inc(step_size);
    }

    pb.finish_and_clear();
}

/* Choose random wallpaper */
fn choose_wallpaper(weather: Weather, wallpapers: &HashSet<Wallpaper>) -> &Wallpaper {
    /* Filter wallpapers by matching day/night */
    let day_filtered: HashSet<&Wallpaper> = wallpapers.iter()
        .filter(|w| w.weather.is_day() == weather.is_day())
        .collect();

    /* Choose random wallpaper */
    match weighted_choice(&weather, &day_filtered) {
        Ok(wallpaper) => wallpaper,
        
        /* No day-appropriate wallpapers - try again with all wallpapers */
        Err(WeightedError::NoItem) => 
            weighted_choice(&weather, &wallpapers.iter().collect::<HashSet<&Wallpaper>>()).unwrap()
        ,

        error => error.unwrap(), /* Too many weights provided or negative weight found */
    }
}

/* Weight wallpapers by number of matching tags, then choose a random wallpaper */
fn weighted_choice<'a>(weather: &Weather, wallpapers: &HashSet<&'a Wallpaper>) -> Result<&'a Wallpaper, WeightedError> {
    let mut rng = rand::thread_rng();

    /* Weight wallpapers by matching tags and favourites */
    let weighted: Vec<(usize, &&Wallpaper)> = wallpapers.iter()
        .map(|wallpaper| (wallpaper.get_weight(&weather), wallpaper)) 
        .collect();

    /* Choose random wallpaper */
    match WeightedIndex::new(weighted.iter().map(|item| item.0)) {
        /* Get wallpaper from vec */
        Ok(dist) => Ok(weighted[dist.sample(&mut rng)].1),
        /* Or choose equally-weighted random wallpaper */
        Err(WeightedError::AllWeightsZero) => Ok(rand_choice(wallpapers)),
        Err(err) => Err(err),
    }
}

/* Choose equally weighted random choice */
fn rand_choice<'a>(wallpapers: &HashSet<&'a Wallpaper>) -> &'a Wallpaper {
    let mut rng = rand::thread_rng();
    wallpapers.iter().choose(&mut rng).expect("No wallpapers found")
}
