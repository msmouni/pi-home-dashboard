use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use reqwest;
use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uuid::Uuid;

const DB_FILE: &str = "/var/lib/pi-home-sensors_data/data.db";
const PI_HOME_DASHBOARD_RES: &str = "/usr/share/pi-home-dashboard/templates";

const SESSION_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Clone, Debug)]
struct UserSession {
    username: String,
    session_start: SystemTime,
}

#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, UserSession>>>, // session_id → UserSession
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct SensorData {
    timestamp: String,
    bmp280_temp: f32,
    bmp280_pressure: f32,
    htu21d_temp: f32,
    htu21d_humidity: f32,
}

#[derive(Serialize)]
struct Weather {
    external_temp: f32,
    external_windspeed: f32,
    external_time: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/data", get(get_data))
        .route("/external-weather", get(external_weather))
        .route("/login", get(show_login).post(handle_login))
        .with_state(state);

    // Run app, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<AppState>, jar: CookieJar) -> Html<String> {
    if let Some(session_id) = jar.get("session_id") {
        let sessions = state.sessions.lock().unwrap();
        if let Some(user_session) = sessions.get(session_id.value()) {
            if user_session.session_start.elapsed().unwrap().as_secs() > SESSION_TIMEOUT_SECS {
                state.sessions.lock().unwrap().remove(session_id.value());
                return Html("<h1>Session expired. <a href='/login'>Login again</a></h1>".into());
            } else {
                let html =
                    std::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/index.html")).unwrap();
                return Html(html);
            }
        }
    }

    Html("<h1>You are not logged in. <a href='/login'>Login</a></h1>".into())
}

async fn get_data() -> Json<Vec<SensorData>> {
    let conn = Connection::open(DB_FILE).unwrap();
    let mut sensors = Vec::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT timestamp, bmp280_temperature, bmp280_pressure, htu21d_temperature, htu21d_humidity \
         FROM SensorData ORDER BY timestamp DESC",
    ) {
        if let Ok(sensor_iter) = stmt.query_map([], |row| {
            Ok(SensorData {
                timestamp: row.get(0)?,
                bmp280_temp: row.get(1)?,
                bmp280_pressure: row.get(2)?,
                htu21d_temp: row.get(3)?,
                htu21d_humidity: row.get(4)?,
            })
        }) {
            for sensor in sensor_iter {
                if let Ok(sensor) = sensor {
                    sensors.push(sensor);
                }
            }
        }
    }

    Json(sensors)
}

async fn external_weather() -> Json<Weather> {
    let url =
        "https://api.open-meteo.com/v1/forecast?latitude=48.85&longitude=2.35&current_weather=true";

    match reqwest::get(url).await {
        Ok(response) if response.status().is_success() => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                let weather = &json["current_weather"];
                let weather_data = Weather {
                    external_temp: weather["temperature"].as_f64().unwrap_or(0.0) as f32,
                    external_windspeed: weather["windspeed"].as_f64().unwrap_or(0.0) as f32,
                    external_time: weather["time"].as_str().unwrap_or("N/A").to_string(),
                };
                return Json(weather_data);
            }
        }
        _ => {}
    }

    // Fallback: return default weather data if any step fails
    Json(Weather {
        external_temp: 0.0,
        external_windspeed: 0.0,
        external_time: "N/A".to_string(),
    })
}

async fn show_login() -> Html<String> {
    let html = tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/login.html"))
        .await
        .unwrap_or_else(|_| "<h1>Login page missing</h1>".into());
    Html(html)
}

async fn handle_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> (CookieJar, Redirect) {
    // TODO: replace with DB check
    if form.username == "admin" && form.password == "raspberry" {
        let session_id = Uuid::new_v4().to_string();

        state.sessions.lock().unwrap().insert(
            session_id.clone(),
            UserSession {
                username: form.username,
                session_start: SystemTime::now(),
            },
        );

        let jar = jar.add(axum_extra::extract::cookie::Cookie::new(
            "session_id",
            session_id,
        ));

        (jar, Redirect::to("/"))
    } else {
        (jar, Redirect::to("/login"))
    }
}
