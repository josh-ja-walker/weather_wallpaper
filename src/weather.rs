use std::{collections::{HashMap, HashSet}, fmt::{self, Display}};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::weather_api::{self, WeatherData};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Weather {
    tags: HashSet<WeatherTag>, 
    is_day: Option<bool>,
}

impl Weather {
    pub fn is_day(&self) -> Option<bool> {
        self.is_day
    }

    pub fn set_is_day(&mut self, is_day: Option<bool>) {
        self.is_day = is_day
    }

    pub fn tags(&self) -> &HashSet<WeatherTag> {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: HashSet<WeatherTag>) {
        self.tags = tags;
    }
}

/* Default weather */
impl Default for Weather {
    fn default() -> Self {
        Self { 
            tags: HashSet::new(), 
            is_day: Some(true)
        }
    }
}

/* Print weather conditions */
impl Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", 
            if self.tags.is_empty() {
                String::from("none").dimmed()
            } else {
                self.tags.iter()
                    .map(WeatherTag::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
                    .bold()
            },

            match self.is_day {
                Some(true) => "day",
                Some(false) => "night",
                None => "day/night",
            },
        )
    }
}

/* Get current Weather status */
pub fn get_current_weather() -> Weather {
    let weather_data: WeatherData = weather_api::fetch_weather_data()
        .expect("Could not fetch weather data from WeatherAPI.com");

    Weather::from(weather_data)
}

/* Adapt WeatherData to Weather */
impl From<WeatherData> for Weather {
    fn from(data: WeatherData) -> Weather {
        Weather {
            tags: WeatherTag::parse(data.text())
                .expect("Could not parse current weather"),
            is_day: Option::from(data.is_day())
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
pub enum WeatherTag { 
    Sun,
    PartCloud,
    Cloud,
    Rain,
    Storm,
    Fog,
    Snow,
}

impl Display for WeatherTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.synonyms().join(", ").to_lowercase())
    }
}

impl WeatherTag {
    /* Adapt and parse WeatherAPI condition to WeatherCond */
    fn parse(cond_text: String) -> Result<HashSet<WeatherTag>, String> {
        let cond_map = load_conditions_map().unwrap();
        
        Ok(cond_map.get(&cond_text)
        .ok_or("Could not find condition in weather_conditions.json")?
        .into_iter()
        .cloned()
        .collect::<HashSet<WeatherTag>>())
    }
    
    /* Synonyms for outputting */
    pub fn synonyms(&self) -> Vec<String> {
        match self {
            WeatherTag::Sun => vec!["Sun", "Clear"],
            WeatherTag::Rain => vec!["Rain", "Drizzle"],
            WeatherTag::Cloud => vec!["Cloudy", "Overcast"],
            WeatherTag::PartCloud => vec!["Partly Cloudy"],
            WeatherTag::Fog => vec!["Mist", "Fog"],
            WeatherTag::Storm => vec!["Stormy", "Thunder"],
            WeatherTag::Snow => vec!["Snow", "Blizzard"],
        }.into_iter()
        .map(String::from)
        .collect()
    }
}

/* Load all conditions from json file */
fn load_conditions_map() -> std::io::Result<HashMap<String, Vec<WeatherTag>>> {
    let contents = include_str!("weather_conditions.json");
    let config: HashMap<String, Vec<WeatherTag>> = serde_json::from_str(&contents)?;
    Ok(config)
}