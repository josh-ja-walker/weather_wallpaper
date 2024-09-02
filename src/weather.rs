use std::{collections::HashSet, fs, io};

use reqwest;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{Weather, WeatherTag};

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    location: Location,
    current: Current,
}

#[derive(Debug, Serialize, Deserialize)]
struct Location {
    name: String,
    region: String,
    country: String,
    lat: f32,
    lon: f32,
    tz_id: String,
    localtime_epoch: usize,
    localtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Current {
    last_updated: String,
    temp_c: f32,
    is_day: u8,

    condition: Condition,
    
    wind_kph: f32,
    precip_mm: f32,
    humidity: u16,
    cloud: u16,
    vis_km: f32,
    uv: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Condition {
    text: String,
    icon: String,
    code: u32,
}


/* Fetch weather data from the API */
#[tokio::main]
async fn fetch_weather_data() -> Result<WeatherData, reqwest::Error> {
    reqwest::Client::new()
        .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=auto:ip")
        // .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=london")
        .send()
        .await?
        .json()
        .await
}

/* Get current Weather status */
pub fn get_current_weather() -> Weather {
    let weather_data: WeatherData = fetch_weather_data()
        .expect("Could not fetch weather data from WeatherAPI.com");

    Weather::from(weather_data)
}

/* Adapt WeatherData to Weather */
impl From<WeatherData> for Weather {
    fn from(data: WeatherData) -> Weather {
        Weather {
            tags: WeatherTag::parse(data.current.condition),
            is_day: data.current.is_day != 0
        }
    }
}

/* Synonyms for parsing weather conditions from WeatherAPI data */
impl WeatherTag {
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

    pub fn to_string(&self) -> String {
        self.synonyms().join(", ").to_lowercase()
    }

    fn matching_codes(&self, conds: &Vec<Condition>) -> Vec<u32> {
        let matching_indexes = match self {
            WeatherTag::Sun => vec![0],
            WeatherTag::PartCloud => vec![1],
            WeatherTag::Cloud => vec![2, 3, 5, 6, 9, 14, 18, 44],
            WeatherTag::Rain => vec![
                5, 7, 8, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 
                24, 25, 26, 27, 35, 36, 37, 38, 39, 44, 45
            ],
            WeatherTag::Storm => vec![9, 44, 45, 46, 47],
            WeatherTag::Fog => vec![4, 12, 13],
            WeatherTag::Snow => vec![
                6, 7, 8, 10, 11, 13, 16, 17, 24, 25, 26, 27, 28, 29, 
                30, 31, 32, 33, 34, 38, 39, 40, 41, 42, 43, 46, 47
            ],
        };

        matching_indexes.into_iter()
            .map(|i| conds[i].code)
            .collect()
    }

    /* Adapt and parse WeatherAPI condition to WeatherCond */
    fn parse(cond: Condition) -> HashSet<WeatherTag> {
        let all_conds = load_all_conditions().unwrap();
        
        WeatherTag::iter()
            .filter(|tag| tag.matching_codes(&all_conds).contains(&cond.code))
            .collect()
    }
}

/* Load all conditions from json file */
fn load_all_conditions() -> io::Result<Vec<Condition>> {
    let contents = fs::read_to_string("weather_conditions.json")?;
    let config: Vec<Condition> = serde_json::from_str(&contents)?;
    Ok(config)
}