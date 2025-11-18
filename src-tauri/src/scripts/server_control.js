(function() {
    if (window.__SERVER_LISTENER_Active__) return;
    window.__SERVER_LISTENER_Active__ = true;

    console.log("🔌 API Server Control loaded");

    window.__TAURI__.event.listen('request-server-port', async () => {
        const portInput = prompt("To START server, enter port (e.g. 3000).\nTo STOP, leave empty and click OK.", "3000");
        
        if (portInput !== null) { 
            let port = parseInt(portInput);
            
            let payload = null;
            if (!isNaN(port) && port > 0) {
                payload = port;
            }

            try {
                const message = await window.__TAURI__.core.invoke('cmd_toggle_server', { port: payload });
                alert("Server Status: " + message);
            } catch (e) {
                alert("Error: " + e);
            }
        }
    });
})();