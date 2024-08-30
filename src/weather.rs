use reqwest;
use serde::{Deserialize, Serialize};

use strum::IntoEnumIterator;

use crate::{Weather, WeatherCond};

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
            condition: WeatherCond::from(data.current.condition),
            is_day: data.current.is_day != 0
        }
    }
}

/* Synonyms for parsing weather conditions from WeatherAPI data */
impl WeatherCond {
    fn synonyms(&self) -> Vec<&str> {
        //TODO: complete synonyms, look at weather_conditions.json
        match self {
            WeatherCond::Clear => vec!["Clear"],
            WeatherCond::Sun => vec!["Sun"],
            WeatherCond::Rain => vec!["Rain, Drizzle"],
            WeatherCond::Cloud => vec!["Cloudy", "Overcast"],
            WeatherCond::PartCloud => vec!["Partly Cloudy"],
            WeatherCond::Fog => vec!["Mist", "Fog"],
            WeatherCond::Storm => vec!["Stormy", "Thunder"],
            WeatherCond::Snow => vec!["Snow", "Blizzard"],
        }
    }
}

/* Adapt and parse WeatherAPI condition to WeatherCond */
impl From<Condition> for WeatherCond {
    fn from(data_cond: Condition) -> Self {
        WeatherCond::iter()
            .filter(|weather_cond: &WeatherCond|
                weather_cond
                    .synonyms()
                    .iter()
                    .any(|syn| data_cond.text.to_lowercase().contains(&syn.to_lowercase())))
            .next().unwrap() //TODO: handle multiple conditions or ensure mutual exclusivity
    }
}