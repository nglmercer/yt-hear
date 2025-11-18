class YouTubeMusicObserver {
  constructor() {
    this.currentSong = null;
    this.pollInterval = null;
    this.videoObserver = null;
    this.lastUrl = "";
    this.isInitialized = false;
    this.lastTimeUpdate = 0;
    this.log = window.Logger; // Acceso rápido al logger

    this.checkChanges = this.checkChanges.bind(this);
  }

  start() {
    if (this.isInitialized) {
        this.log.warn("Intento de iniciar Observer ya inicializado.");
        return;
    }
    this.isInitialized = true;

    // Verificar dependencia CRÍTICA
    if (!window.YTM || !window.YTM.Info || typeof window.YTM.Info.get !== 'function') {
        this.log.error("CRÍTICO: window.YTM.Info.get no está definido. El script 'songinfo.js' no se cargó o tiene el nombre incorrecto.");
    }

    this.observeVideoLifecycle();
    this.startSmartPolling();
    this.observeMetadataChanges();
    this.observeUrlChanges();

    this.log.info("Iniciado correctamente (Messaging Mode)");
    this.checkChanges(); 
  }

  observeVideoLifecycle() {
    const attach = (v) => {
        this.log.debug("Elemento <video> detectado, adjuntando listeners...");
        this.attachVideoListeners(v);
    };
    
    const video = document.querySelector("video");
    if (video) attach(video);
    else this.log.warn("No se encontró <video> al inicio. Esperando inyección...");

    this.videoObserver = new MutationObserver((mutations) => {
      for (const m of mutations) {
        for (const node of m.addedNodes) {
          if (node.tagName === "VIDEO") {
              this.log.info("Nuevo elemento <video> inyectado en el DOM");
              attach(node);
          }
        }
      }
    });
    this.videoObserver.observe(document.body, { childList: true, subtree: true });
  }

  attachVideoListeners(video) {
    if (!video || video.dataset.ytmObserverAttached) return;
    video.dataset.ytmObserverAttached = "true";

    // Debug: Saber cuándo ocurren eventos nativos
    const events = ["play", "pause", "ended", "loadeddata", "durationchange"];
    events.forEach((evt) => {
        video.addEventListener(evt, (e) => {
            this.log.debug(`Evento nativo video: ${evt}`);
            this.checkChanges();
        });
    });

    video.addEventListener("timeupdate", () => {
      const now = Date.now();
      if (now - this.lastTimeUpdate > 1000) {
        this.lastTimeUpdate = now;
        this.notifyTimeUpdate(video);
      }
    });
  }

  startSmartPolling() {
    this.pollInterval = setInterval(() => {
      // Verificación de salud
      if (location.href !== this.lastUrl) {
        this.log.debug("Polling: URL cambió");
        this.checkChanges();
      } else if (document.title !== this.currentSong?.title) {
        // A veces el título del documento cambia antes que el DOM interno
        this.checkChanges();
      }
    }, 2000);
  }

  observeMetadataChanges() {
    const playerBar = document.querySelector("ytmusic-player-bar");
    if (!playerBar) {
        this.log.warn("No se encontró 'ytmusic-player-bar' para observar metadatos");
        return;
    }

    const observer = new MutationObserver(() => {
      if (this._domDebounce) clearTimeout(this._domDebounce);
      this._domDebounce = setTimeout(() => {
          // this.log.debug("DOM Mutation en PlayerBar detectada");
          this.checkChanges();
      }, 500);
    });

    observer.observe(playerBar, { subtree: true, characterData: true, childList: true });
  }

  observeUrlChanges() {
    const originalPushState = history.pushState.bind(history);
    history.pushState = (...args) => {
      originalPushState(...args);
      this.log.debug("History PushState detectado");
      this.checkChanges();
    };
    window.addEventListener("popstate", () => {
        this.log.debug("PopState detectado");
        this.checkChanges();
    });
  }

  checkChanges() {
    // Validación de seguridad
    if (!window.YTM?.Info?.get) return;

    const newInfo = window.YTM.Info.get();
    if (!newInfo) {
        this.log.debug("YTM.Info.get() devolvió null (¿Publicidad o carga?)");
        return;
    }

    this.lastUrl = location.href;

    // Comparación detallada para debug
    let changes = [];
    if (!this.currentSong) changes.push("First Run");
    else {
        if (newInfo.songDuration === 0 && typeof newInfo.album !== 'string') return;
        if (newInfo.title !== this.currentSong.title) changes.push(`Title: ${this.currentSong.title} -> ${newInfo.title}`);
        if (newInfo.isPaused !== this.currentSong.isPaused) changes.push(`State: ${this.currentSong.isPaused ? 'Paused' : 'Playing'} -> ${newInfo.isPaused ? 'Paused' : 'Playing'}`);
        if (Math.abs(newInfo.songDuration - this.currentSong.songDuration) > 2) changes.push("Duration changed");
        // Añadir más condiciones si es necesario
    }

    const hasChanged = changes.length > 0;

    if (hasChanged) {
      this.log.info("Cambio detectado:", changes.join(", "));
      this.currentSong = newInfo;
      this.broadcast("song-info", newInfo);
    }
  }

  notifyTimeUpdate(video) {
    if (!this.currentSong) return;
    // Opcional: Debuguear tiempo (puede ser muy ruidoso)
    // this.log.debug(`Time: ${video.currentTime}`);
    
    this.broadcast("time-tick", {
        elapsedSeconds: Math.floor(video.currentTime),
        songDuration: Math.floor(video.duration || 0)
    });
  }

  broadcast(type, data) {
    try {
        window.postMessage({
            source: 'pear-wrapper',
            event: type,
            payload: data
        }, window.location.origin);
    } catch (e) {
        this.log.error("Error enviando postMessage:", e);
    }
  }

  stop() {
    if (this.pollInterval) clearInterval(this.pollInterval);
    if (this.videoObserver) this.videoObserver.disconnect();
    this.isInitialized = false;
    this.log.info("Observer detenido");
  }
}

// Inicialización con control de duplicados
window.YTM = window.YTM || {};

// Detener instancia anterior si existe (útil para Hot Reloading)
if (window.YTM.Observer && typeof window.YTM.Observer.stop === 'function') {
    console.log("[YTM] Deteniendo instancia anterior...");
    window.YTM.Observer.stop();
}

if (location.hostname.includes("music.youtube.com")) {
    const waitForApp = setInterval(() => {
        if (document.querySelector("ytmusic-player-bar") && document.querySelector("video")) {
            clearInterval(waitForApp);
            
            // Instanciamos y guardamos en el namespace
            window.YTM.Observer = new YouTubeMusicObserver();
            window.YTM.Observer.start();
        }
    }, 500);
}