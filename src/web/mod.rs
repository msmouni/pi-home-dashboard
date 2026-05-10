mod routes;

use crate::{
    state::AppState,
    web::routes::{external_weather, get_data, handle_login, index, logout, show_login},
};
use axum::{routing::get, Router};
use std::io::Error;
use tokio::net::TcpListener;

const PI_HOME_DASHBOARD_PORT: u16 = 3000;

pub struct App {
    pub service: Router,
    pub listener: TcpListener,
}

pub async fn get_app() -> Result<App, Error> {
    let state = AppState::default();

    let service = Router::new()
        .route("/", get(index))
        .route("/data", get(get_data))
        .route("/external-weather", get(external_weather))
        .route("/login", get(show_login).post(handle_login))
        .route("/logout", get(logout))
        .with_state(state);

    // Run app, listening globally on port PI_HOME_DASHBOARD_PORT
    tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PI_HOME_DASHBOARD_PORT))
        .await
        .map(|listener| App { service, listener })
}
