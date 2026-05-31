use rusqlite::Connection;
use serde::Serialize;

#[allow(dead_code)]
const DB_FILE: &str = "/var/lib/pi-home-sensors_data/data.db";

#[derive(Serialize, Debug, Default, Clone)]
pub struct SensorData {
    pub timestamp: String,
    pub bmp280_temp: f32,
    pub bmp280_pressure: f32,
    pub htu21d_temp: f32,
    pub htu21d_humidity: f32,
}

#[allow(dead_code)]
pub fn get_sensors_data_from_db() -> Vec<SensorData> {
    let conn = Connection::open(DB_FILE).unwrap();
    let mut sensors = Vec::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT timestamp, bmp280_temperature, bmp280_pressure, htu21d_temperature, htu21d_humidity \
         FROM SensorData ORDER BY timestamp DESC",
    ) {
        if let Ok(sensor_iter) = stmt.query_map([], |row| {
            Ok(SensorData {
                timestamp: row.get(0)?,
                bmp280_temp: row.get(1)?,
                bmp280_pressure: row.get(2)?,
                htu21d_temp: row.get(3)?,
                htu21d_humidity: row.get(4)?,
            })
        }) {
            for sensor in sensor_iter {
                if let Ok(sensor) = sensor {
                    sensors.push(sensor);
                }
            }
        }
    }

    sensors
}
