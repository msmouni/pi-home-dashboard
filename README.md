# Pi-Home Dashboard

[Pi-Home Dashboard](https://github.com/msmouni/pi-home-dashboard) is the web application for the [Pi-Home](https://github.com/msmouni/pi-home-os) system, built with Rust and [Axum](https://github.com/tokio-rs/axum).

It provides user authentication and a web interface for monitoring sensor data and controlling connected devices.

Sensor data collected by [pi-home-sensors](https://github.com/msmouni/pi-home-sensors) is stored in [SQLite](https://sqlite.org/) and displayed in the web application with graphical visualizations.

The dashboard also communicates with MQTT devices:

* **Zigbee devices** are connected through [Zigbee2MQTT](https://www.zigbee2mqtt.io/) and a Zigbee dongle.
* **MCU-based devices**, such as those running [sensors-node](https://github.com/msmouni/sensors-node), communicate through MQTT over Wi-Fi.

Both sensor information and device commands are exposed through the web interface, allowing users to monitor their environment and control connected devices from a browser.
