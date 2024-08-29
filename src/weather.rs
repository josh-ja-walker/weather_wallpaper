use reqwest;
use serde::{Deserialize, Serialize};

use crate::{Wallpaper, Weather, WeatherCond};

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
        .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=auto:ip&aqi=no")
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
            condition: todo!(), // data.current.condition.text,
            is_day: data.current.is_day == 0
        }
    }
}

