use chrono::Local;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::Value;
use std::time::Duration;

use crate::{sensors::SensorData, state::AppState, zigbee::PlugDevice};

const MQTT_BROKER: &str = "localhost";
const MQTT_PORT: u16 = 1883;

pub async fn permit_join(client: &AsyncClient) -> Result<(), rumqttc::ClientError> {
    client
        .publish(
            "zigbee2mqtt/bridge/request/permit_join",
            QoS::AtMostOnce,
            false,
            r#"{"value":true,"time":60}"#,
        )
        .await
}

pub async fn request_devices(client: &AsyncClient) -> Result<(), rumqttc::ClientError> {
    client
        .publish(
            "zigbee2mqtt/bridge/request/devices",
            QoS::AtMostOnce,
            false,
            "{}",
        )
        .await
}

pub async fn turn_on(client: &AsyncClient, device: &str) {
    client
        .publish(
            format!("zigbee2mqtt/{}/set", device),
            QoS::AtMostOnce,
            false,
            r#"{"state":"ON"}"#,
        )
        .await
        .unwrap();
}

pub async fn turn_off(client: &AsyncClient, device: &str) {
    client
        .publish(
            format!("zigbee2mqtt/{}/set", device),
            QoS::AtMostOnce,
            false,
            r#"{"state":"OFF"}"#,
        )
        .await
        .unwrap();
}

#[allow(dead_code)]
pub async fn request_state(client: &AsyncClient, device: &str) {
    client
        .publish(
            format!("zigbee2mqtt/{}/get", device),
            QoS::AtMostOnce,
            false,
            r#"{}"#,
        )
        .await
        .unwrap();
}

fn handle_bridge_devices(app_state: &AppState, payload: &str) {
    let Ok(json) = serde_json::from_str::<Value>(payload) else {
        return;
    };

    let Some(array) = json.as_array() else {
        return;
    };

    let mut devices = app_state.zigbee_devices.lock().unwrap();

    for item in array {
        if let Some(dev_type) = item.get("type").and_then(|v| v.as_str()) {
            /* Ignore Coordinator devices */
            if dev_type == "Coordinator" {
                continue;
            }
        }

        let Some(friendly_name) = item.get("friendly_name").and_then(|v| v.as_str()) else {
            continue;
        };

        devices
            .entry(friendly_name.to_string())
            .or_insert(PlugDevice {
                id: friendly_name.to_string(),
                name: friendly_name.to_string(),

                state: None,
                power: None,
                voltage: None,
                current: None,
            });
    }
}

fn handle_device_update(app_state: &AppState, device_id: &str, payload: &str) {
    let Ok(json) = serde_json::from_str::<Value>(payload) else {
        return;
    };

    let mut devices = app_state.zigbee_devices.lock().unwrap();

    let device = devices.entry(device_id.to_string()).or_insert(PlugDevice {
        id: device_id.to_string(),
        name: device_id.to_string(),

        state: None,
        power: None,
        voltage: None,
        current: None,
    });

    if let Some(v) = json.get("power").and_then(|v| v.as_f64()) {
        device.power = Some(v as f32);
    }

    if let Some(v) = json.get("voltage").and_then(|v| v.as_f64()) {
        device.voltage = Some(v as f32);
    }

    if let Some(v) = json.get("current").and_then(|v| v.as_f64()) {
        device.current = Some(v as f32);
    }

    if let Some(v) = json.get("state").and_then(|v| v.as_str()) {
        device.state = Some(v == "ON");
    }
}

fn handle_zigbee_devices(app_state: &AppState, topic: &str, payload: &str) {
    // Device discovery topic - update device list
    if topic == "zigbee2mqtt/bridge/devices" {
        handle_bridge_devices(app_state, payload);
        return;
    }

    // Ignore other bridge topics (states, logs ...)
    if topic.starts_with("zigbee2mqtt/bridge/") {
        return;
    }

    // Device update topic - update device state
    let parts: Vec<&str> = topic.split('/').collect();
    /*
    Consider zigbee2mqtt/<device>
    Ignore:
        zigbee2mqtt/<device>/set
        zigbee2mqtt/<device>/get
        zigbee2mqtt/<device>/availability
    */
    if parts.len() == 2 && parts[0] == "zigbee2mqtt" {
        let device_id = parts[1];

        handle_device_update(app_state, device_id, payload);
    }
}

fn handle_wifi_devices(app_state: &AppState, _device_id: &str, payload: &str) {
    let Ok(json) = serde_json::from_str::<Value>(payload) else {
        return;
    };

    app_state.sensor_data.lock().unwrap().push(SensorData {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        bmp280_temp: json
            .get("bmp_temp")
            .and_then(|bmp_temp| bmp_temp.as_f64())
            .unwrap_or_default() as f32,
        bmp280_pressure: json
            .get("bmp_press")
            .and_then(|bmp_pressure| bmp_pressure.as_f64())
            .unwrap_or_default() as f32,
        htu21d_temp: json
            .get("htu_temp")
            .and_then(|htu_temp| htu_temp.as_f64())
            .unwrap_or_default() as f32,
        htu21d_humidity: json
            .get("htu_hum")
            .and_then(|htu_humidity| htu_humidity.as_f64())
            .unwrap_or_default() as f32,
    });
}

pub async fn mqtt_setup(app_state: AppState) {
    let mut mqttoptions = MqttOptions::new("rust-client", MQTT_BROKER, MQTT_PORT);

    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    /* Subscribe to zigbee2mqtt sensors */
    client
        .subscribe("zigbee2mqtt/#", QoS::AtMostOnce)
        .await
        .unwrap();

    /* Subscribe to WIFI/mqtt sensors */
    client
        .subscribe("sensors/#", QoS::AtMostOnce)
        .await
        .unwrap();

    app_state
        .mqtt_client
        .lock()
        .unwrap()
        .replace(client.clone());

    println!("MQTT listening...");

    // Request device list on startup
    client
        .publish(
            "zigbee2mqtt/bridge/request/devices",
            QoS::AtMostOnce,
            false,
            "{}",
        )
        .await
        .unwrap();

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    match p.topic {
                        topic if topic.starts_with("zigbee2mqtt/") => {
                            // handle Zigbee2MQTT messages
                            let payload = String::from_utf8_lossy(&p.payload);

                            handle_zigbee_devices(&app_state, &topic, &payload);
                        }

                        topic if topic.starts_with("sensors/") => {
                            // handle ESP32 sensors
                            let payload = String::from_utf8_lossy(&p.payload);

                            handle_wifi_devices(&app_state, &topic, &payload);
                        }

                        _ => {}
                    }
                }

                Ok(_) => {}

                Err(e) => {
                    println!("MQTT error: {:?}", e);

                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });
}
