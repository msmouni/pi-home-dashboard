function fetchExternalWeather() {
    fetch("/external-weather")
        .then((response) => response.json())
        .then((data) => {
            document.getElementById("external-weather").innerHTML = `
                              <strong>External Weather:</strong><br>
                  🌡️ ${data.external_temp} °C<br>
                  💨 ${data.external_windspeed} km/h<br>
                  🕒 ${data.external_time}
                          `;
        });
}