use std::{collections::{HashMap, HashSet}, fmt::{self, Display}};

use reqwest;
use serde::{Deserialize, Serialize};

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
            tags: WeatherTag::parse(data.current.condition.text)
                .expect("Could not parse condition"),
            is_day: data.current.is_day != 0
        }
    }
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
    let contents = include_str!("../weather_conditions.json");
    let config: HashMap<String, Vec<WeatherTag>> = serde_json::from_str(&contents)?;
    Ok(config)
}