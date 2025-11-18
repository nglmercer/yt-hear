// scripts/bridge.js

window.Pear = window.Pear || {};

window.Pear.Bridge = {
    send: (topic, data) => sendTelemetry(topic, data)
};

function sendTelemetry(topic, data) {
    if (!window.__TAURI__?.core) {
        console.log("!__TAURI__");
        return;
    }
    if (window.__TAURI__?.core) {
        console.log("sendTelemetry", topic, data);
        window.__TAURI__.core.invoke('push_telemetry', { topic, payload: data });
    }
}
window.addEventListener("message", (event) => {
    if (event.origin !== window.location.origin) {
        console.log("Message origin", event.origin, "vs", window.location.origin);
        return;
    }
    const msg = event.data;
    if (!msg || msg.source !== 'pear-wrapper') {
        console.log("Message source", msg.source, "vs", 'pear-wrapper');
        return;
    }
    sendTelemetry(msg.event, msg.payload);
});
// src-tauri/src/scripts/providers/controller.js (o el archivo correspondiente)

(function() {
    // Asegurarse de que estamos en el contexto de Tauri
    if (window.__TAURI__) {
        const { listen } = window.__TAURI__.event;
        console.log("🎧 Listening for External Commands...");
        // Escuchamos el evento que definimos en Rust ("ytm:command")
        listen('ytm:command', (event) => {
            const command = event.payload;
            console.log("📨 Received Command:", command);

            handleCommand(command);
        });
    }

    function handleCommand(data) {
        switch (data.action) {
            case 'play':
                window.YTM.Player.play();
                break;
            case 'pause':
                window.YTM.Player.pause();
                break;
            case 'playPause':
                window.YTM.Player.playPause();
                break;
            case 'next':
                window.YTM.Player.next();
                break;
            case 'previous':
                window.YTM.Player.previous();
                break;
            case 'seek':
                // data.value en segundos
                if (typeof data.value === 'number') window.YTM.Player.seekTo(data.value);
                break;
            // ... otros comandos
        }
    }
})();