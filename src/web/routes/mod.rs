mod auth;
mod data;
mod index;
mod weather;
mod zigbee;

pub use auth::{handle_login, logout, show_login};
pub use data::get_data;
pub use index::index;
pub use weather::external_weather;
pub use zigbee::{zigbee_get_devices, zigbee_permit_join, zigbee_refresh, zigbee_toggle};

const PI_HOME_DASHBOARD_TEMPLATES: &str = "/usr/share/pi-home-dashboard/templates";
pub(super) const PI_HOME_DASHBOARD_STATIC: &str = "/usr/share/pi-home-dashboard/static";
