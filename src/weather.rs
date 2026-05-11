use serde::Serialize;

#[derive(Serialize)]
pub struct Weather {
    external_temp: f32,
    external_windspeed: f32,
    external_time: String,
}

impl Default for Weather {
    fn default() -> Self {
        Weather {
            external_temp: 0.0,
            external_windspeed: 0.0,
            external_time: "N/A".to_string(),
        }
    }
}

pub async fn get_external_weather() -> Option<Weather> {
    let url =
        "https://api.open-meteo.com/v1/forecast?latitude=48.85&longitude=2.35&current_weather=true";

    match reqwest::get(url).await {
        Ok(response) if response.status().is_success() => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                let weather = &json["current_weather"];
                return Some(Weather {
                    external_temp: weather["temperature"].as_f64().unwrap_or(0.0) as f32,
                    external_windspeed: weather["windspeed"].as_f64().unwrap_or(0.0) as f32,
                    external_time: weather["time"].as_str().unwrap_or("N/A").to_string(),
                });
            }
        }
        _ => {}
    }

    None
}
