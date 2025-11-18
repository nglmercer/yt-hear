class YouTubeMusicListeners {
  constructor() {
    this.observers = [];
    // Ya no necesitamos this.callbacks ni on()
    
    this.currentVideo = null;
    this.videoHandlers = {}; 
  }

  init() {
    this.observeVideoLifecycle();
    this.observePlayerBarState();
    console.log("[YTM Listeners] Iniciado via postMessage");
  }
  observeVideoLifecycle() {
    const attachToVideo = (videoNode) => {
      if (!videoNode || this.currentVideo === videoNode) return;

      // Limpieza previa
      if (this.currentVideo) {
        this.currentVideo.removeEventListener("timeupdate", this.videoHandlers.time);
        this.currentVideo.removeEventListener("volumechange", this.videoHandlers.volume);
        this.currentVideo.removeEventListener("seeked", this.videoHandlers.seek);
      }

      this.currentVideo = videoNode;

      // Handlers que emiten mensajes
      this.videoHandlers.time = () => {
        this.broadcast("time-update", {
          currentTime: videoNode.currentTime,
          duration: videoNode.duration || 0,
        });
      };

      this.videoHandlers.volume = () => {
        this.broadcast("volume-change", {
          volume: Math.round(videoNode.volume * 100),
          isMuted: videoNode.muted,
        });
      };

      this.videoHandlers.seek = () => {
        // Seek dispara time-update, pero a veces queremos saber explícitamente que hubo un salto
        this.broadcast("seeked", {
           currentTime: videoNode.currentTime
        });
        this.videoHandlers.time();
      };

      videoNode.addEventListener("timeupdate", this.videoHandlers.time);
      videoNode.addEventListener("volumechange", this.videoHandlers.volume);
      videoNode.addEventListener("seeked", this.videoHandlers.seek);
    };

    const initialVideo = document.querySelector("video");
    if (initialVideo) attachToVideo(initialVideo);

    const observer = new MutationObserver((mutations) => {
      for (const m of mutations) {
        for (const node of m.addedNodes) {
          if (node.tagName === "VIDEO") attachToVideo(node);
        }
      }
    });

    observer.observe(document.body, { childList: true, subtree: true });
    this.observers.push(observer);
  }

  observePlayerBarState() {
    const waitForBar = setInterval(() => {
      const playerBar = document.querySelector("ytmusic-player-bar");
      if (!playerBar) return;

      clearInterval(waitForBar);

      const observer = new MutationObserver(() => {
        // Lógica de detección (basada en tu código anterior)
        const isShuffleOn = playerBar.hasAttribute("shuffle-on");
        const isFullscreen = playerBar.hasAttribute("player-fullscreened");
        
        // Detección mejorada de repeat (híbrida: atributo nuevo o botones viejos)
        let repeatMode = playerBar.getAttribute("repeat-mode") || "NONE";
        
        if (!playerBar.hasAttribute("repeat-mode")) {
             // Fallback para versiones viejas
             const repeatBtn = playerBar.querySelector(".repeat");
             if (repeatBtn) {
                const title = (repeatBtn.title || repeatBtn.getAttribute("aria-label") || "").toLowerCase();
                if (title.includes("una") || title.includes("one")) repeatMode = "ONE";
                else if (title.includes("todo") || title.includes("all")) repeatMode = "ALL";
             }
        }

        this.broadcast("state-change", {
          shuffle: isShuffleOn,
          fullscreen: isFullscreen,
          repeat: repeatMode,
        });
      });

      observer.observe(playerBar, {
        attributes: true,
        subtree: true,
        attributeFilter: ["shuffle-on", "player-fullscreened", "repeat-mode", "title", "aria-label"],
      });

      this.observers.push(observer);
    }, 1000);
  }

  /**
   * Envía un mensaje seguro a la ventana.
   * @param {string} type - El tipo de evento (ej: 'time-update')
   * @param {object} data - Los datos
   */
  broadcast(type, data) {
    const message = {
        source: 'pear-wrapper',
        event: type,
        payload: data
    };
    window.postMessage(message, window.location.origin);
  }

  destroy() {
    this.observers.forEach((obs) => obs.disconnect());
    if (this.currentVideo) {
      this.currentVideo.removeEventListener("timeupdate", this.videoHandlers.time);
      this.currentVideo.removeEventListener("volumechange", this.videoHandlers.volume);
      this.currentVideo.removeEventListener("seeked", this.videoHandlers.seek);
    }
    this.observers = [];
  }
}

// Inicialización
window.YTM = window.YTM || {};
window.YTM.Events = new YouTubeMusicListeners();
// window.YTM.Events.init(); // Descomentar si quieres auto-iniciar o iniciarlo desde bridge.js