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