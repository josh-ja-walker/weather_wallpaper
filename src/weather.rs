use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WeatherAPI {
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
    last_updated: String,
    temp_c: f32,
    is_day: u8,

    condition: Condition,

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

#[tokio::main]
async fn get_weather() -> Result<WeatherAPI, reqwest::Error> 
{
    let weather_api: WeatherAPI = reqwest::Client::new()
    .get("http://api.weatherapi.com/v1/current.json?key=d89f01f4ac164824b2c194551221707&q=auto:ip&aqi=no")
        .send()
        .await?
        .json()
        .await?;

    Ok(weather_api)
}

pub fn set_tags(weather_tags: &mut Vec<&str>) 
{
    let weather_api = get_weather().unwrap();
    
    let mut is_clear: bool = true;

    if weather_api.current.vis_km <= 1.0 {
        weather_tags.push("fog");
    }

    if weather_api.current.wind_kph > 25.0 {
        weather_tags.push("wind");
    }

    if weather_api.current.temp_c < 3.0 
    {
        weather_tags.push("cold");
    }
    else if weather_api.current.temp_c > ( if weather_api.current.is_day != 0 {25.0} else {21.0} ) 
    {
        weather_tags.push("hot");
    }
    
    if weather_api.current.cloud > 20
    {
        if weather_api.current.cloud < 60 
        {
            weather_tags.push("part_cl");
        } 
        else 
        {
            weather_tags.push("cloud");
        }

        is_clear = false;
    }

    if weather_api.current.precip_mm > 0.8 
    {
        weather_tags.push("rain");
        is_clear = false;
    }

    if weather_api.current.is_day != 0 
    {
        if weather_api.current.uv > 3.75 
        {
            weather_tags.push("sun");
            is_clear = false;
        }
    }
    else 
    {
        weather_tags.push("night");
    }

    if is_clear 
    {
        weather_tags.push("clear");
    }
    
    println!("\nCurrent Weather Tags: ");
    for tag in weather_tags 
    {
        println!("\t- {}", crate::title(tag));
    }
    println!();

}