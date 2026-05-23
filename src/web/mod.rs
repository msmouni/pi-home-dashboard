mod routes;

use crate::{
    state::AppState,
    web::routes::{
        external_weather, get_data, handle_login, index, logout, show_login, zigbee_get_devices,
        zigbee_permit_join, zigbee_refresh, zigbee_toggle, PI_HOME_DASHBOARD_STATIC,
    },
};
use axum::{
    routing::{get, post},
    Router,
};
use std::io::Error;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

const PI_HOME_DASHBOARD_PORT: u16 = 3000;

pub struct App {
    pub service: Router,
    pub listener: TcpListener,
}

pub async fn get_app(state: AppState) -> Result<App, Error> {
    let service = Router::new()
        .route("/", get(index))
        .route("/zigbee/devices", get(zigbee_get_devices))
        .route("/zigbee/refresh", post(zigbee_refresh))
        .route("/zigbee/permit_join", post(zigbee_permit_join))
        .route("/zigbee/{id}/toggle", post(zigbee_toggle))
        .route("/data", get(get_data))
        .route("/external-weather", get(external_weather))
        .route("/login", get(show_login).post(handle_login))
        .route("/logout", get(logout))
        .nest_service("/static", ServeDir::new(PI_HOME_DASHBOARD_STATIC))
        .with_state(state);

    // Run app, listening globally on port PI_HOME_DASHBOARD_PORT
    tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PI_HOME_DASHBOARD_PORT))
        .await
        .map(|listener| App { service, listener })
}
