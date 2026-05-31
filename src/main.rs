mod sensors;
mod state;
mod utils;
mod weather;
mod web;
mod zigbee;

use crate::{state::AppState, web::App, zigbee::mqtt_setup};

#[tokio::main]
async fn main() {
    println!("Starting Pi Home Dashboard...");

    let state = AppState::default();

    let App {
        service: app,
        listener,
    } = web::get_app(state.clone()).await.unwrap();

    mqtt_setup(state).await;

    if let Ok(local_addr) = listener.local_addr() {
        println!("Server running at http://{}", local_addr);
    }

    axum::serve(listener, app).await.unwrap();
}
