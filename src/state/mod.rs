pub(crate) mod session;

use crate::{sensors::SensorData, utils::CircularBuffer, zigbee::PlugDevice};
use rumqttc::AsyncClient;
use session::UserSession;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uuid::Uuid;

const SENSORS_HISTORY_CAPACITY: usize = 500;

#[derive(Clone, Debug, Default)]
pub(crate) struct AppState {
    sessions: Arc<Mutex<HashMap<String, UserSession>>>, // session_id → UserSession

    pub mqtt_client: Arc<Mutex<Option<AsyncClient>>>,
    pub zigbee_devices: Arc<Mutex<HashMap<String, PlugDevice>>>, // device_id → Device
    pub sensor_data: Arc<Mutex<CircularBuffer<SensorData, SENSORS_HISTORY_CAPACITY>>>, // TODO: handle multiple sensors
}

pub fn state_try_login(
    state: &mut AppState,
    username: &String,
    password: &String,
) -> Option<String> {
    // TODO: replace with DB check
    if username == "admin" && password == "raspberry" {
        let session_id = Uuid::new_v4().to_string();

        state.sessions.lock().unwrap().insert(
            session_id.clone(),
            UserSession {
                username: username.clone(),
                session_start: SystemTime::now(),
            },
        );

        Some(session_id)
    } else {
        None
    }
}

pub fn state_logout(state: &mut AppState, session_id: &String) {
    state.sessions.lock().unwrap().remove(session_id);
}
