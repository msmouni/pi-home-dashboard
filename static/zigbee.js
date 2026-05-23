async function fetchZigbeeDevices() {
    const res = await fetch("/zigbee/devices");
    const devices = await res.json();

    const container = document.getElementById("zigbee-devices");

    container.innerHTML = "";

    devices.forEach((device) => {
        container.innerHTML += `
                <div class="bg-gray-700 rounded-lg p-4 shadow">
                  <div class="flex justify-between items-center mb-2">
                    <h5 class="text-lg font-bold">${device.name}</h5>

                    <button
                      onclick="toggleDevice('${device.id}')"
                      class="${device.state
                ? "bg-red-600 hover:bg-red-700"
                : "bg-green-600 hover:bg-green-700"
            } text-white px-3 py-1 rounded"
                    >
                      ${device.state ? "TURN OFF" : "TURN ON"}
                    </button>
                  </div>

                  <div class="space-y-1 text-sm text-gray-300">
                    <p>⚡ Power: ${device.power} W</p>
                    <p>🔌 Voltage: ${device.voltage} V</p>
                    <p>🔋 Current: ${(device.current / 100).toFixed(2)} A</p>
                  </div>
                </div>
              `;
    });
}

async function toggleDevice(id) {
    await fetch(`/zigbee/${id}/toggle`, {
        method: "POST",
    });

    fetchZigbeeDevices();
}

async function refreshDevices() {
    const status = document.getElementById("zigbee-status");

    status.innerText = "Refreshing devices...";

    try {
        const response = await fetch("/zigbee/refresh", {
            method: "POST",
        });

        const text = await response.text();

        status.innerText = text;

        fetchZigbeeDevices();
    } catch (err) {
        status.innerText = "Failed to refresh devices";
    }

    // Clear message after 5 seconds
    setTimeout(() => {
        status.innerText = "";
    }, 5000);
}

async function permitJoin() {
    const status = document.getElementById("zigbee-status");

    status.innerText = "Enabling permit join...";

    try {
        const response = await fetch("/zigbee/permit_join", {
            method: "POST",
        });

        const text = await response.text();

        status.innerText = text;
    } catch (err) {
        status.innerText = "Failed to enable permit join";
    }

    // Clear message after 5 seconds
    setTimeout(() => {
        status.innerText = "";
    }, 5000);
}