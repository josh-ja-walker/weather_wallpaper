use std::{
    thread, time::Duration,
    collections::HashSet, 
    fmt::Display, 
};

use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use dialoguer::Select;
use colored::Colorize;

use rand::{prelude::*, distributions::WeightedIndex};

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
    InvalidWallpaper,
    ImagePrintFail,
    Interrupted,
    InvalidInput,
}

fn main() {
    let mut config = settings::load_settings().unwrap_or_default();

    if files::load_all_wallpapers().is_empty() {
        println!("Weather Wallpaper:");
        println!("No wallpapers found. Add wallpapers to {}", files::wallpapers_path().unwrap().to_str().unwrap());
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
            1 => wallpaper_tags::edit_wallpaper_tags(),
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
    loop {
        Term::stdout().clear_screen().unwrap();
        println!("{}", "Weather Wallpaper:".bold());

        let curr_weather: Weather = weather::get_current_weather();
        println!("Current Weather: {}", curr_weather);
    
        print!("Chosen: ");
        let chosen: Wallpaper = choose_wallpaper(curr_weather, files::load_all_wallpapers());
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
fn choose_wallpaper(weather: Weather, wallpapers: HashSet<Wallpaper>) -> Wallpaper {
    let mut rng = thread_rng();
    
    let mut day_filtered: Vec<&Wallpaper> = wallpapers
        .iter()
        .filter(|w| w.weather.is_day() == weather.is_day())
        .collect();

    if day_filtered.is_empty() {
        day_filtered = wallpapers.iter().collect();
    }
    
    let tag_weighted: Vec<(usize, &&Wallpaper)> = day_filtered
        .iter()
        .map(|w| (w.weather.tags().intersection(weather.tags()).count(), w)) 
        .filter(|(num_match, _)| *num_match > 0)
        .collect();

    if tag_weighted.is_empty() {
        day_filtered
            .into_iter()
            .choose(&mut rng)
            .unwrap()
    } else {
        let dist = WeightedIndex::new(
            tag_weighted
                .iter()
                .map(|item| item.0)
            ).unwrap();

        tag_weighted[dist.sample(&mut rng)].1
    }.clone()
}
