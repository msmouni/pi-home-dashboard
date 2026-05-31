use crate::{
    sensors::SensorData,
    state::{session::verify_session, AppState},
};
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;

pub async fn get_data(State(state): State<AppState>, jar: CookieJar) -> Json<Vec<SensorData>> {
    if !verify_session(&state, jar) {
        return Json(Vec::new());
    }

    Json(
        state
            .sensor_data
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .collect(),
    )
}
