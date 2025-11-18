class YouTubeMusicStateProvider {
  constructor() {
    this.playerBar = null;
    this.video = null;
    this.likeRenderer = null;
    this._refresh();
  }

  /** Actualiza las referencias al DOM */
  _refresh() {
    this.playerBar = document.querySelector("ytmusic-player-bar");
    this.video = document.querySelector("video");
    this.likeRenderer = document.querySelector("ytmusic-like-button-renderer");
  }

  /**
   * Obtiene el estado del Shuffle (Aleatorio)
   * @returns {boolean}
   */
  getShuffleState() {
    this._refresh();
    if (!this.playerBar) return false;

    // 1. Método más fiable: Atributo en la barra principal
    // En tu HTML no aparece activo, pero cuando se activa, YTM añade 'shuffle-on'
    if (this.playerBar.hasAttribute("shuffle-on")) return true;

    // 2. Fallback: Botón específico
    const shuffleBtn = this.playerBar.querySelector(".shuffle");
    if (shuffleBtn) {
      // El botón interno <button> tiene aria-pressed="true" si está activo
      const innerBtn = shuffleBtn.querySelector("button");
      if (innerBtn?.getAttribute("aria-pressed") === "true") return true;
    }

    return false;
  }

  /**
   * Obtiene el estado de Repetición
   * Basado en el atributo 'repeat-mode' de tu HTML: <ytmusic-player-bar ... repeat-mode="NONE">
   * @returns {'NONE' | 'ONE' | 'ALL'}
   */
  getRepeatState() {
    this._refresh();
    if (!this.playerBar) return "NONE";

    // ¡Mucho mejor! Leemos el atributo directo del componente
    const mode = this.playerBar.getAttribute("repeat-mode");
    if (mode) return mode; // Devuelve "NONE", "ONE", o "ALL" directamente

    return "NONE";
  }

  /**
   * Obtiene Like/Dislike
   * Basado en el atributo 'like-status' de tu HTML: <ytmusic-like-button-renderer ... like-status="LIKE">
   * @returns {'LIKE' | 'DISLIKE' | 'INDIFFERENT'}
   */
  getLikeStatus() {
    this._refresh();

    // 1. Método directo: Atributo del componente renderer
    if (this.likeRenderer) {
      const status = this.likeRenderer.getAttribute("like-status");
      if (status) return status; // Devuelve "LIKE", "DISLIKE", etc.
    }

    // 2. Fallback: Botones individuales (por si el atributo falla)
    const likeBtn = this.likeRenderer?.querySelector(".like button");
    const dislikeBtn = this.likeRenderer?.querySelector(".dislike button");

    if (likeBtn?.getAttribute("aria-pressed") === "true") return "LIKE";
    if (dislikeBtn?.getAttribute("aria-pressed") === "true") return "DISLIKE";

    return "INDIFFERENT";
  }

  /**
   * Obtiene el estado de Pantalla Completa
   * @returns {boolean}
   */
  getFullscreenState() {
    this._refresh();

    // 1. Verificación nativa del navegador (Infalible)
    if (document.fullscreenElement) return true;

    // 2. Verificación de atributo YTM (según tu HTML, no estaba presente en el snippet, pero se añade al activar)
    if (this.playerBar?.hasAttribute("player-fullscreened")) return true;

    return false;
  }

  /**
   * Obtiene volumen y mute
   * Preferimos el <video> source, pero usamos el slider del HTML como backup
   */
  getVolumeState() {
    this._refresh();

    // Opción A: Elemento de video real (Audio real)
    if (this.video) {
      return {
        volume: Math.round(this.video.volume * 100),
        isMuted: this.video.muted,
      };
    }

    // Opción B: Lectura del Slider del HTML (Visual)
    // Tu HTML: <tp-yt-paper-slider id="volume-slider" ... aria-valuenow="5">
    const slider = document.querySelector("#volume-slider");
    if (slider) {
      const vol = parseInt(slider.getAttribute("aria-valuenow") || "50");
      // Es difícil saber si está muteado solo con el slider, asumimos false
      return { volume: vol, isMuted: false };
    }

    return { volume: 50, isMuted: false };
  }

  /**
   * Devuelve todo el estado de una vez
   */
  getAllState() {
    return {
      shuffle: this.getShuffleState(),
      repeat: this.getRepeatState(), // Ahora devuelve "NONE", "ONE", "ALL" limpio
      fullscreen: this.getFullscreenState(),
      volume: this.getVolumeState(),
      likeStatus: this.getLikeStatus(), // Ahora devuelve "LIKE", "DISLIKE", "INDIFFERENT" limpio
    };
  }
}

window.YTM = window.YTM || {};
window.YTM.State = new YouTubeMusicStateProvider();