class YouTubeMusicControls {
  constructor() {
    this.video = null;
    this.playerBar = null;
    this.isInitialized = false;
  }

  /** Inicializa esperando a que existan los elementos */
  async init() {
    if (this.isInitialized && document.body.contains(this.video)) return true;
    await this.waitForPlayer();
    this.isInitialized = true;
    return true;
  }

  waitForPlayer() {
    return new Promise((resolve) => {
      let attempts = 0;
      const check = () => {
        attempts++;
        this.video = document.querySelector("video");
        this.playerBar = document.querySelector("ytmusic-player-bar");

        if (this.video && this.playerBar) {
          resolve();
        } else {
          // Timeout de 10 segundos para no quedarse en loop infinito
          if (attempts > 20) {
            console.warn("YTM Controls: No se encontró el reproductor.");
            resolve(); // Resolvemos igual para no romper la app, pero sin video
            return;
          }
          setTimeout(check, 500);
        }
      };
      check();
    });
  }

  /**
   * Función maestra para hacer clic.
   * Busca en el DOM normal y dentro del Shadow DOM del reproductor.
   */
  clickButton(selectors) {
    // 1. Intentar encontrar el botón en el DOM global
    for (const sel of selectors) {
      const btn = document.querySelector(sel);
      if (btn && !btn.disabled && btn.offsetParent !== null) {
        // offsetParent verifica que sea visible
        btn.click();
        return true;
      }
    }

    // 2. Intentar encontrar dentro del Shadow DOM del player-bar
    if (this.playerBar && this.playerBar.shadowRoot) {
      for (const sel of selectors) {
        const btn = this.playerBar.shadowRoot.querySelector(sel);
        if (btn && !btn.disabled) {
          btn.click();
          return true;
        }
      }
    }

    console.warn(`[YTM] No se encontró botón para: ${selectors[0]}`);
    return false;
  }

  // ========================
  // CONTROLES DE REPRODUCCIÓN
  // ========================

  async play() {
    if (!this.video) await this.init();
    return this.video?.play();
  }

  async pause() {
    if (!this.video) await this.init();
    this.video?.pause();
  }

  async playPause() {
    if (!this.video) await this.init();
    if (!this.video) return;

    if (this.video.paused) await this.video.play();
    else this.video.pause();
  }

  // NOTA: Usamos selectores de CLASE o ID, no de texto (title/aria-label)
  // para que funcione en Inglés, Español, etc.
  next() {
    return this.clickButton([
      ".next-button", // Clase estándar
      "#left-controls .next-button", // ID específico
      "tp-yt-paper-icon-button.next-button", // Web Component
    ]);
  }

  previous() {
    return this.clickButton([
      ".previous-button",
      "#left-controls .previous-button",
      "tp-yt-paper-icon-button.previous-button",
    ]);
  }

  // ========================
  // SEEK & TIEMPO
  // ========================

  seekTo(seconds) {
    if (!this.video) return false;
    const duration = this.video.duration || 0;
    this.video.currentTime = Math.max(0, Math.min(seconds, duration));
    return true;
  }

  goForward(seconds = 10) {
    if (!this.video) return false;
    this.video.currentTime = Math.min(
      this.video.currentTime + seconds,
      this.video.duration || 0,
    );
    return true;
  }

  goBack(seconds = 10) {
    if (!this.video) return false;
    this.video.currentTime = Math.max(this.video.currentTime - seconds, 0);
    return true;
  }

  // ========================
  // VOLUMEN
  // ========================

  setVolume(level) {
    if (!this.video) return false;
    // Asegurar rango 0-100
    const vol = Math.max(0, Math.min(100, level));
    this.video.volume = vol / 100;
    if (vol > 0) this.video.muted = false;

    // Intentar actualizar el slider visual (es puramente cosmético)
    // Es complejo actualizar el slider de Polymer desde fuera,
    // pero cambiar el video.volume es lo que realmente importa.
    try {
      const slider = document.querySelector("#volume-slider");
      if (slider) slider.value = vol;
    } catch (e) {}

    return true;
  }

  getVolume() {
    return this.video ? Math.round(this.video.volume * 100) : 0;
  }

  toggleMute() {
    if (!this.video) return false;
    this.video.muted = !this.video.muted;
    return this.video.muted;
  }

  // ========================
  // ACCIONES SOCIALES (Like/Shuffle)
  // ========================

  like() {
    // El botón de like suele estar dentro de un renderer específico
    return this.clickButton([
      // Selector moderno basado en iconos path (más seguro que texto)
      // Pero como los paths cambian, usamos la estructura de clases
      ".like-button-renderer tp-yt-paper-icon-button.like",
      "ytmusic-like-button-renderer .like",
      // Fallback a texto multilingüe si falla la clase
      'button[aria-label="Me gusta"]',
      'button[aria-label="Like"]',
    ]);
  }

  dislike() {
    return this.clickButton([
      ".like-button-renderer tp-yt-paper-icon-button.dislike",
      "ytmusic-like-button-renderer .dislike",
      'button[aria-label="No me gusta"]',
      'button[aria-label="Dislike"]',
    ]);
  }

  shuffle() {
    // El botón shuffle está en los controles de la derecha
    return this.clickButton([
      ".shuffle",
      "#right-controls .shuffle",
      "tp-yt-paper-icon-button.shuffle",
    ]);
  }

  switchRepeat() {
    // El botón repeat también está a la derecha
    return this.clickButton([
      ".repeat",
      "#right-controls .repeat",
      "tp-yt-paper-icon-button.repeat",
    ]);
  }

  // ========================
  // ESTADO
  // ========================

  isPlaying() {
    return this.video ? !this.video.paused && !this.video.ended : false;
  }

  getStatus() {
    if (!this.video) return "IDLE";
    return {
      isPlaying: !this.video.paused,
      time: Math.floor(this.video.currentTime),
      duration: Math.floor(this.video.duration),
      volume: Math.floor(this.video.volume * 100),
    };
  }
}

window.YTM = window.YTM || {};
window.YTM.Player = new YouTubeMusicControls();

// Inicialización segura
if (location.hostname.includes("music.youtube.com")) {
    const init = () => window.YTM.Player.init();
    if (document.readyState === "complete" || document.readyState === "interactive") {
        init();
    } else {
        window.addEventListener("DOMContentLoaded", init);
    }
}