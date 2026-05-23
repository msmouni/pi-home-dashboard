google.charts.load("current", { packages: ["corechart"] });
google.charts.setOnLoadCallback(initialize);

function initialize() {
    fetchAll();
    setInterval(fetchAll, 5000); // update every 5s
}

function fetchAll() {
    fetchSensorData();
    fetchExternalWeather();
    fetchZigbeeDevices();
}
