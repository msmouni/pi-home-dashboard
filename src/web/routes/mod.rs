mod auth;
mod data;
mod index;
mod weather;

pub use auth::{handle_login, logout, show_login};
pub use data::get_data;
pub use index::index;
pub use weather::external_weather;

const PI_HOME_DASHBOARD_RES: &str = "/usr/share/pi-home-dashboard/templates";
