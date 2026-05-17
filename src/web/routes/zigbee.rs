use crate::{
    state::{session::verify_session, AppState},
    zigbee::{turn_off, turn_on, PlugDevice},
};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::CookieJar;

pub async fn zigbee_get_devices(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Json<Vec<PlugDevice>> {
    if !verify_session(&state, jar) {
        return Json(Vec::new());
    }

    let mut devices = Vec::new();

    for device in state.zigbee_devices.lock().unwrap().values() {
        devices.push(device.clone());
    }

    Json(devices)
}

pub async fn zigbee_permit_join(State(state): State<AppState>, jar: CookieJar) -> &'static str {
    if !verify_session(&state, jar) {
        return "Unauthorized";
    }

    let opt_client = state.mqtt_client.lock().unwrap().clone();

    if let Some(client) = opt_client {
        let result = crate::zigbee::permit_join(&client).await;
        match result {
            Ok(_) => "Permit join enabled for 60 seconds",
            Err(_) => "Failed to enable permit join",
        }
    } else {
        "MQTT client not initialized"
    }
}

pub async fn zigbee_refresh(State(state): State<AppState>, jar: CookieJar) -> &'static str {
    if !verify_session(&state, jar) {
        return "Unauthorized";
    }

    let client = state.mqtt_client.lock().unwrap().clone();

    match client {
        Some(client) => {
            let result = crate::zigbee::request_devices(&client).await;

            match result {
                Ok(_) => "Device refresh requested",
                Err(_) => "Failed to refresh devices",
            }
        }
        None => "MQTT client not initialized",
    }
}

pub async fn zigbee_toggle(
    Path(id): Path<String>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> &'static str {
    if !verify_session(&state, jar) {
        return "Unauthorized";
    }

    let device_info = {
        let devices = state.zigbee_devices.lock().unwrap();

        devices
            .get(&id)
            .map(|device| (device.name.clone(), device.state.unwrap_or(false)))
    };

    let opt_client = state.mqtt_client.lock().unwrap().clone();

    if let Some((device_name, device_state)) = device_info {
        if let Some(client) = opt_client {
            if device_state {
                turn_off(&client, &device_name).await;
                "Turning off"
            } else {
                turn_on(&client, &device_name).await;
                "Turning on"
            }
        } else {
            "MQTT client not initialized"
        }
    } else {
        "Device not found"
    }
}
