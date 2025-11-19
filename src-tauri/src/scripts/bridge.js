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
(function() {
    // Verificar contexto Tauri
    if (!window.__TAURI__) return;

    const { listen } = window.__TAURI__.event;
    const { invoke } = window.__TAURI__.core;
    console.log("🎧 YTM Controller: Listening for Rust commands...");

    listen('ytm:command', (event) => {
        const cmd = event.payload;
        console.log("📨 Command received:", cmd);
        
        if (!cmd || !cmd.action) return;

        try {
            switch (cmd.action) {
                // --- Player Controls ---
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
                
                // --- Time / Seek ---
                case 'seek':
                    // cmd.value es segundos
                    window.YTM.Player.seekTo(cmd.value);
                    break;
                case 'goBack':
                    window.YTM.Player.goBack(cmd.value || 10);
                    break;
                case 'goForward':
                    window.YTM.Player.goForward(cmd.value || 10);
                    break;

                // --- Volume ---
                case 'setVolume':
                    window.YTM.Player.setVolume(cmd.value);
                    break;
                case 'toggleMute':
                    window.YTM.Player.toggleMute();
                    break;

                // --- Feedback ---
                case 'like':
                    window.YTM.Player.like();
                    break;
                case 'dislike':
                    window.YTM.Player.dislike();
                    break;

                // --- Queue Management ---
                case 'addToQueue':
                    // Requiere videoId y opcionalmente insertPosition
                    if (cmd.videoId && window.YTM.Queue) {
                        window.YTM.Queue.addToQueue(cmd.videoId, cmd.insertPosition);
                    }
                    break;
                case 'clearQueue':
                    if (window.YTM.Queue) window.YTM.Queue.clearQueue();
                    break;
                case 'removeFromQueue':
                    if (window.YTM.Queue) window.YTM.Queue.removeFromQueue(cmd.value);
                    break;
                case 'setQueueIndex':
                    if (window.YTM.Queue) window.YTM.Queue.setIndex(cmd.value);
                    break;
                case 'moveInQueue':
                    if (window.YTM.Queue && typeof cmd.fromIndex === 'number' && typeof cmd.toIndex === 'number') {
                        window.YTM.Queue.moveInQueue(cmd.fromIndex, cmd.toIndex);
                    }
                    break;

                case 'search':
                    if (cmd.query) {
                        window.YTM.Search.search(cmd.query);
                    }
                    break;
                case 'toggleShuffle':
                    window.YTM.Player.shuffle()
                    break;
                case 'switchRepeat':
                    window.YTM.Player.switchRepeat()
                    break;
                default:
                    console.warn("⚠️ Unknown command action:", cmd.action);
            }
        } catch (e) {
            console.error("❌ Error executing command:", cmd, e);
        }
    });
    listen('ytm:request', async (event) => {
        const { request_id, topic } = event.payload;
        let responseData = null;
        try {
            switch (topic) {
                case 'get-song-info':
                    responseData = await window.YTM?.Info?.get() || null;
                    break;
                
                case 'get-queue':
                    responseData = await window.YTM?.Queue?.getQueueData() || null;
                    break;

                case 'get-volume':
                    responseData = await window.YTM?.State?.getVolumeState() || null;
                    break;

                default:
                    responseData = { error: "Unknown topic" };
            }
        } catch (e) {
            responseData = { error: e.message };
        }
        console.log("responseData", responseData);
        invoke('resolve_request', { 
            requestId: request_id,
            data: responseData 
        }).catch(err => console.error("Error resolving request:", err));
    });
})();