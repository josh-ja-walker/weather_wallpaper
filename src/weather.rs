use std::collections::HashSet;

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
    lat: f64,
    lon: f64,
    tz_id: String,
    localtime_epoch: u128,
    localtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Current {
    condition: Condition,
    is_day: u8,

    last_updated: String,

    temp_c: f32,
    wind_kph: f64,
    precip_mm: f64,
    humidity: i16,
    cloud: i16,
    vis_km: f64,
    uv: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Condition {
    text: String,
    icon: String,
    code: i128,
}


/* Fetch weather data from the API */
#[tokio::main]
async fn fetch_weather_data() -> Result<WeatherData, reqwest::Error> {
    reqwest::Client::new()
        // .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=auto:ip")
        .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=london")
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
        //TODO: complete synonyms, look at weather_conditions.json
        match self {
            WeatherTag::Clear => vec!["Clear"],
            WeatherTag::Sun => vec!["Sun"],
            WeatherTag::Rain => vec!["Rain, Drizzle"],
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

    /* Adapt and parse WeatherAPI condition to WeatherCond */
    fn parse(data_cond: Condition) -> HashSet<WeatherTag> {
        WeatherTag::iter()
            .filter(|weather_cond: &WeatherTag|
                weather_cond
                    .synonyms()
                    .iter() 
                    /* Check for contained synonyms */
                    /* TODO: fix issue with cloudy = partly cloudy */
                    .any(|syn| data_cond.text.to_lowercase().contains(&syn.to_lowercase())))
            .collect()
    }
}