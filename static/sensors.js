function fetchSensorData() {
    fetch("/data")
        .then((response) => response.json())
        .then((data) => drawCharts(data));
}

function drawCharts(data) {
    let tempData = [["Time", "BMP280 Temp (°C)", "HTU21D Temp (°C)"]];
    let pressureData = [["Time", "Pressure (hPa)"]];
    let humidityData = [["Time", "Humidity (%)"]];

    data.reverse().forEach((row) => {
        tempData.push([row.timestamp, row.bmp280_temp, row.htu21d_temp]);
        pressureData.push([row.timestamp, row.bmp280_pressure]);
        humidityData.push([row.timestamp, row.htu21d_humidity]);
    });

    drawChart(tempData, "temp_chart", "Temperature (°C)");
    drawChart(pressureData, "pressure_chart", "Pressure (hPa)");
    drawChart(humidityData, "humidity_chart", "Humidity (%)");
}

function drawChart(dataArray, elementId, vAxisTitle) {
    const data = google.visualization.arrayToDataTable(dataArray);
    const options = {
        curveType: "function",
        legend: { position: "bottom", textStyle: { color: "#d1d5db" } },
        vAxis: {
            title: vAxisTitle,
            textStyle: { color: "#d1d5db" },
            titleTextStyle: { color: "#d1d5db" },
        },
        hAxis: { textStyle: { color: "#d1d5db" } },
        backgroundColor: "#1f2937", // dark background
        titleTextStyle: { color: "#e5e7eb" },

        // Tooltip style
        tooltip: {
            textStyle: { color: "#111827" }, // Tailwind gray-900 (dark text)
            showColorCode: true,
            isHtml: true,
        },
    };
    const chart = new google.visualization.LineChart(
        document.getElementById(elementId),
    );
    chart.draw(data, options);
}
