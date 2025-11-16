use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use reqwest::{self, header::CACHE_CONTROL};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uuid::Uuid;

const DB_FILE: &str = "/var/lib/pi-home-sensors_data/data.db";
const PI_HOME_DASHBOARD_RES: &str = "/usr/share/pi-home-dashboard/templates";

const SESSION_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Clone, Debug)]
struct UserSession {
    #[allow(dead_code)]
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
        .route("/logout", get(logout))
        .with_state(state);

    // Run app, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn verify_session(state: AppState, jar: CookieJar) -> bool {
    if let Some(session_id) = jar.get("session_id") {
        let sessions = state.sessions.lock().unwrap();
        if let Some(user_session) = sessions.get(session_id.value()) {
            if user_session.session_start.elapsed().unwrap().as_secs() > SESSION_TIMEOUT_SECS {
                state.sessions.lock().unwrap().remove(session_id.value());
                return false;
            } else {
                return true;
            }
        }
    }

    false
}

async fn index(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if verify_session(state, jar).await {
        let html = tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/index.html"))
            .await
            .unwrap();
        Html(html).into_response()
    } else {
        let mut response = Html(
            tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/not_logged_in.html"))
                .await
                .unwrap(),
        )
        .into_response();

        response.headers_mut().insert(
            CACHE_CONTROL,
            "no-cache, no-store, must-revalidate".parse().unwrap(),
        );

        response
    }
}

async fn get_data(State(state): State<AppState>, jar: CookieJar) -> Json<Vec<SensorData>> {
    if !verify_session(state, jar).await {
        return Json(Vec::new());
    }

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

async fn external_weather(State(state): State<AppState>, jar: CookieJar) -> Json<Weather> {
    if !verify_session(state, jar).await {
        return Json(Weather {
            external_temp: 0.0,
            external_windspeed: 0.0,
            external_time: "N/A".to_string(),
        });
    }

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

async fn show_login(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if verify_session(state, jar).await {
        Redirect::to("/").into_response()
    } else {
        let html = tokio::fs::read_to_string(format!("{PI_HOME_DASHBOARD_RES}/login.html"))
            .await
            .unwrap_or_else(|_| "<h1>Login page missing</h1>".into());
        let mut response = Html(html).into_response();

        response.headers_mut().insert(
            CACHE_CONTROL,
            "no-cache, no-store, must-revalidate".parse().unwrap(),
        );

        response
    }
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

        let cookie = Cookie::build(("session_id", session_id.clone()))
            .max_age(time::Duration::seconds(SESSION_TIMEOUT_SECS as i64))
            .http_only(true); // more secure: not accessible from JS

        let jar = jar.add(cookie);

        (jar, Redirect::to("/"))
    } else {
        (jar, Redirect::to("/login?error=1"))
    }
}

async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, Redirect) {
    if let Some(session_cookie) = jar.get("session_id") {
        let session_id = session_cookie.value().to_string();
        state.sessions.lock().unwrap().remove(&session_id);
    }

    // Expire cookie immediately
    let expired = Cookie::build(("session_id", ""))
        .path("/")
        .max_age(time::Duration::seconds(0));

    let jar = jar.add(expired);
    (jar, Redirect::to("/login"))
}
