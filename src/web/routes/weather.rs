use crate::state::session::verify_session;
use crate::weather::{get_external_weather, Weather};
use crate::web::AppState;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;

pub async fn external_weather(State(state): State<AppState>, jar: CookieJar) -> Json<Weather> {
    if !verify_session(&state, jar) {
        return Json(Weather::default());
    }

    if let Some(weather) = get_external_weather().await {
        Json(weather)
    } else {
        Json(Weather::default())
    }
}
