/**
 * Esta clase DEBE ejecutarse en el contexto de la página (MAIN WORLD),
 * no en el content script aislado de la extensión.
 */
class YouTubeMusicQueueController {
  constructor() {
    this.queueEl = null;
    this.appEl = null;
    this.init();
  }

  init() {
    this.queueEl = document.querySelector("#queue");
    this.appEl = document.querySelector("ytmusic-app");

    if (!this.queueEl || !this.appEl) {
      console.warn(
        "YTM Queue Controller: Esperando a que cargue la interfaz...",
      );
      return false;
    }
    return true;
  }

  // ==========================================
  // LECTURA (Read)
  // ==========================================

  getQueueData() {
    if (!this.init()) return null;

    // Accedemos a la API interna que vimos en tu console.dir
    // queueEl.queue es un getter que devuelve la instancia del componente
    const internalQueue = this.queueEl.queue;

    if (!internalQueue) return null;

    return {
      items: internalQueue.getItems(), // Función nativa de YTM
      autoPlaying: internalQueue.autoPlaying,
      continuation: internalQueue.continuation,
      selectedIndex: this._getSelectedIndex(internalQueue),
    };
  }

  _getSelectedIndex(internalQueue) {
    // Recuperamos el estado desde el store de Redux interno
    const store = internalQueue.store?.store?.getState();
    if (!store) return -1;

    // Buscamos cuál tiene el flag 'selected'
    return store.queue.items.findIndex(
      (it) =>
        it.playlistPanelVideoRenderer?.selected ||
        it.playlistPanelVideoWrapperRenderer?.primaryRenderer
          ?.playlistPanelVideoRenderer?.selected,
    );
  }

  // ==========================================
  // ESCRITURA (Write) - La parte difícil
  // ==========================================

  /**
   * Añadir canción requiere obtener sus metadatos del servidor de Google primero.
   * Usamos 'networkManager' que viene en <ytmusic-app>
   */
  async addToQueue(videoId, position = "INSERT_AT_END") {
    // O 'INSERT_AFTER_CURRENT_VIDEO'
    if (!this.init()) return;

    try {
      // 1. Necesitamos el contexto actual de la cola para pedir la nueva canción
      const store = this.queueEl.queue.store.store;
      const state = store.getState();

      // 2. Hacemos la petición interna a la API de YTM
      // Esto es magia negra: usamos el propio cliente de red de YouTube
      const response = await this.appEl.networkManager.fetch(
        "/music/get_queue",
        {
          queueContextParams: state.queue.queueContextParams,
          queueInsertPosition: position,
          videoIds: [videoId],
        },
      );

      if (!response || !response.queueDatas)
        throw new Error("No data received");

      // 3. Preparamos los items (limpiamos la respuesta)
      const newItems = response.queueDatas
        .map((it) => it.content)
        .filter(Boolean);

      // 4. Calculamos dónde insertar
      const currentItems = state.queue.items;
      let insertIndex = currentItems.length;

      if (position === "INSERT_AFTER_CURRENT_VIDEO") {
        const currentIndex = this._getSelectedIndex(this.queueEl.queue);
        insertIndex = currentIndex + 1;
      }

      // 5. DISPATCH: La acción de Redux que actualiza la UI
      // Usamos el dispatch del elemento DOM que vimos en tu consola
      this.queueEl.dispatch({
        type: "ADD_ITEMS",
        payload: {
          nextQueueItemId: state.queue.nextQueueItemId,
          index: insertIndex,
          items: newItems,
          shuffleEnabled: false,
          shouldAssignIds: true,
        },
      });

      console.log(`✅ Añadido a la cola: ${videoId}`);
      return true;
    } catch (e) {
      console.error("❌ Error en addToQueue:", e);
      return false;
    }
  }

  removeFromQueue(index) {
    if (!this.init()) return;
    // Acción directa de Redux descubierta en el código fuente
    this.queueEl.dispatch({
      type: "REMOVE_ITEM",
      payload: index,
    });
  }

  moveInQueue(fromIndex, toIndex) {
    if (!this.init()) return;
    this.queueEl.dispatch({
      type: "MOVE_ITEM",
      payload: { fromIndex, toIndex },
    });
  }

  clearQueue() {
    if (!this.init()) return;

    // Primero cerramos el player visualmente
    this.queueEl.queue.store.store.dispatch({
      type: "SET_PLAYER_PAGE_INFO",
      payload: { open: false },
    });

    // Luego borramos datos
    this.queueEl.dispatch({ type: "CLEAR" });
  }

  setIndex(index) {
    if (!this.init()) return;
    this.queueEl.dispatch({
      type: "SET_INDEX",
      payload: index,
    });
  }
}

window.YTM = window.YTM || {};
window.YTM.Queue = new YouTubeMusicQueueController();
