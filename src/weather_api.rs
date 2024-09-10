use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    location: Location,
    current: Current,
}

impl WeatherData {
    pub fn is_day(&self) -> bool {
        self.current.is_day == 1
    }

    pub fn text(&self) -> String {
        self.current.condition.text.clone()
    }
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

#[derive(Debug, Serialize, Deserialize)]
struct Condition {
    text: String,
    icon: String,
    code: u32,
}


/* Fetch weather data from the API */
#[tokio::main]
pub async fn fetch_weather_data() -> Result<WeatherData, reqwest::Error> {
    reqwest::Client::new()
        .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=auto:ip")
        .send()
        .await?
        .json()
        .await
}
