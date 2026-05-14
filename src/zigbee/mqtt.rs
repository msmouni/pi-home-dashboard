use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::Value;
use std::time::Duration;

use crate::{state::AppState, zigbee::PlugDevice};

/// From data/configuration.yaml -> device -> friendly_name
// const DEVICE: &str = "0xa4c138d05937aa67";

const MQTT_BROKER: &str = "localhost";
const MQTT_PORT: u16 = 1883;

pub async fn permit_join(client: &AsyncClient) {
    client
        .publish(
            "zigbee2mqtt/bridge/request/permit_join",
            QoS::AtMostOnce,
            false,
            r#"{"value":true,"time":60}"#,
        )
        .await
        .unwrap();
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

pub async fn mqtt_setup(app_state: AppState) {
    let mut mqttoptions = MqttOptions::new("rust-client", MQTT_BROKER, MQTT_PORT);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Also try zigbee2mqtt/#
    client
        .subscribe("zigbee2mqtt/+", QoS::AtMostOnce)
        .await
        .unwrap();

    app_state.mqtt_client.lock().unwrap().replace(client);

    println!("MQTT listening...");

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    if let Some(device_id) = p.topic.strip_prefix("zigbee2mqtt/") {
                        let mut devices = app_state.zigbee_devices.lock().unwrap();

                        let device = devices.entry(device_id.to_string()).or_insert(PlugDevice {
                            id: device_id.to_string(),
                            name: device_id.to_string(),

                            state: None,
                            power: None,
                            voltage: None,
                            current: None,
                        });

                        let payload = String::from_utf8_lossy(&p.payload);

                        if let Ok(json) = serde_json::from_str::<Value>(&payload) {
                            if let Some(v) = json.get("power").and_then(|v| v.as_i64()) {
                                device.power.replace(v as f32);
                            }

                            if let Some(v) = json.get("voltage").and_then(|v| v.as_i64()) {
                                device.voltage.replace(v as f32);
                            }

                            if let Some(v) = json.get("current").and_then(|v| v.as_f64()) {
                                device.current.replace(v as f32);
                            }

                            if let Some(v) = json.get("state").and_then(|v| v.as_str()) {
                                device.state.replace(v == "ON");
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    println!("MQTT error: {:?}", e);
                    break;
                }
            }
        }
    });
}
