mod sensors;
mod state;
mod weather;
mod web;

use crate::web::App;

#[tokio::main]
async fn main() {
    let App {
        service: app,
        listener,
    } = web::get_app().await.unwrap();

    if let Ok(local_addr) = listener.local_addr() {
        println!("Server running at http://{}", local_addr);
    }

    axum::serve(listener, app).await.unwrap();
}
