use std::collections::HashSet;

use console::Term;
use dialoguer::{Input, MultiSelect, Select};
use strum::IntoEnumIterator;

use crate::{Error, files, format_items, wallpaper::{self, Wallpaper}, weather::WeatherTag};


/* Edit the tags of all wallpapers */
pub fn edit_wallpaper_tags() {
    let mut wallpapers = files::load_all_wallpapers()
        .into_iter()
        .collect::<Vec<Wallpaper>>();

    edit_menu(0, &mut wallpapers);

    wallpaper::save_wallpapers(&wallpapers.into_iter().collect()).unwrap();
}

/* Edit the tags of a wallpaper */
fn edit_menu(index: usize, wallpapers: &mut Vec<Wallpaper>) {
    if index >= wallpapers.len() {
        return;
    }

    print!("{}. ", index);

    match wallpapers[index].edit_tags() {
        Ok(_) => edit_menu(index + 1, wallpapers),
        Err(Error::Interrupted) => interrupted_menu(index, wallpapers),
        error => error.unwrap(), 
    }
}

/* Interrupted editing of tags (skip/goto/quit) */
fn interrupted_menu(index: usize, wallpapers: &mut Vec<Wallpaper>) {
    let control = Select::new()
        .with_prompt("Interrupted")
        .items(&format_items(vec![
            "Next",
            "Prev",
            "Go to ",
            "Reset all tags", 
            "Back to menu",
        ]))
        .default(0)
        .report(false)
        .interact()
        .unwrap();

    let new_index = match control {
        0 => index + 1, /* Next */
        1 => index.checked_sub(1).unwrap_or(0), /* Prev */
        2 => goto_menu(wallpapers), /* Goto x */ 
        3 => { /* Clear all tags */
            wallpaper::save_wallpapers(&HashSet::new()).unwrap();

            *wallpapers = files::load_all_wallpapers()
                .into_iter()
                .collect::<Vec<Wallpaper>>();
            
            wallpapers.len()
        },
        4 => wallpapers.len(), /* Quit */
        _ => unreachable!(),
    };

    edit_menu(new_index, wallpapers)
}

/* Handle input for goto */
fn goto_menu(wallpapers: &mut Vec<Wallpaper>) -> usize {
    let goto_index = Input::<usize>::new()
        .with_prompt("Enter index of wallpaper to edit")
        .validate_with(|input: &usize| 
            (*input < wallpapers.len())
                .then_some(())
                .ok_or(Error::InvalidInput)
        )
        .interact()
        .unwrap();

    Term::stdout().clear_last_lines(1).unwrap();
    
    goto_index
}


impl Wallpaper {
    /* Edit the tags of wallpaper */
    fn edit_tags(&mut self) -> Result<(), Error> {
        self.print();
        
        /* Load tag options */
        let options: Vec<(String, bool)> = WeatherTag::iter()
            .map(|tag| (tag.to_string(), self.weather.tags().contains(&tag)))
            .collect();
    
        /* Set what weather is depicted */
        let input = MultiSelect::new()
            .with_prompt("Select weather tags")
            .items_checked(&options)
            .report(false)
            .interact_opt()
            .unwrap()
            .ok_or(Error::Interrupted)?;
        
        /* Set whether day or night is depicted */
        let day_night = Select::new()
            .with_prompt("Select day or night")
            .items(&format_items(vec!["Day", "Night"]))
            .default(!self.weather.is_day() as usize)
            .report(false)
            .interact_opt()
            .unwrap()
            .ok_or(Error::Interrupted)?;
    
        /* Update tags */
        self.weather.set_tags(
            WeatherTag::iter()
                .enumerate()
                .filter_map(|(i, tag)| input.contains(&i).then_some(tag))
                .collect::<HashSet<WeatherTag>>()
        );
    
        /* Update day/night */
        self.weather.set_is_day(day_night == 0);
    
        Ok(())
    }
}

