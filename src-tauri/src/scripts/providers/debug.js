const YTMQueueInspector = {
  /** Imprime el estado real vs el estado visual */
  inspect: () => {
    const queueEl = document.querySelector("ytmusic-player-queue");
    const visualItems = document.querySelectorAll("ytmusic-player-queue-item");

    console.group("🕵️ YTM Queue Inspector");

    // 1. Análisis Visual (DOM)
    console.log(
      `%c👁️ Visible en DOM: ${visualItems.length} items`,
      "color: orange; font-weight: bold",
    );

    // 2. Análisis Interno (Memoria Polymer/WebComponent)
    if (queueEl && queueEl.queue) {
      const internalItems = queueEl.queue.items || [];
      console.log(
        `%c🧠 Real en Memoria: ${internalItems.length} items`,
        "color: lightgreen; font-weight: bold",
      );

      if (internalItems.length > 0) {
        console.log(
          "📦 Estructura del primer item (Raw Data):",
          internalItems[0],
        );

        // Intentar mapear para ver si nuestra lógica funciona
        const mapped = internalItems.map((item, i) => {
          const r = item.playlistPanelVideoRenderer;
          if (!r) return `[${i}] Tipo desconocido`;
          return `[${i}] ${r.title?.runs?.[0]?.text} - ${r.shortBylineText?.runs?.[0]?.text} (ID: ${r.videoId})`;
        });
        console.table(mapped.slice(0, 10)); // Mostrar solo los primeros 10
      }
    } else {
      console.error(
        "❌ No se pudo acceder a la propiedad .queue interna. ¿Google cambió el nombre?",
      );
      console.log("Elemento base:", queueEl);
    }
    console.groupEnd();
  },

  /** Monitor de cambios: Te avisa cuando la cola cambia internamente */
  monitor: () => {
    const queueEl = document.querySelector("ytmusic-player-queue");
    if (!queueEl) return console.error("No queue element");

    console.log("👀 Monitoreando cambios en la cola...");
    // Polymer suele usar eventos 'iron-items-changed' o cambios en propiedades
    // Pero un MutationObserver al elemento puede revelar si cambian atributos de datos
    const observer = new MutationObserver((mutations) => {
      console.log(
        "⚡ DOM de la cola cambió. Ejecuta YTMQueueInspector.inspect() para ver detalles.",
      );
    });
    observer.observe(queueEl, { childList: true, subtree: true });
  },
};
const YTMDataSpy = {
  findPlayerResponse: () => {
    // Estrategia 1: Buscar en el elemento <ytmusic-app>
    const app = document.querySelector("ytmusic-app");
    console.group("🕵️ Data Spy");

    if (app && app.playerResponse) {
      console.log("✅ Found app.playerResponse:", app.playerResponse);
    } else {
      console.log("❌ app.playerResponse not found");
    }

    // Estrategia 2: Buscar en el elemento de navegación
    const navigator = document.querySelector("ytmusic-app-layout");
    if (navigator && navigator.playerResponse) {
      console.log("✅ Found layout.playerResponse:", navigator.playerResponse);
    }

    // Estrategia 3: Interceptar la API (La más avanzada)
    console.log(
      "💡 Tip: Si no encuentras datos, revisa la red (Network Tab) buscando 'next' o 'player'.",
    );
    console.groupEnd();
  },
};
const YTMActionSimulator = {
  highlightButtons: () => {
    const buttons = document.querySelectorAll(
      "button, tp-yt-paper-icon-button",
    );
    buttons.forEach((btn) => {
      btn.style.border = "2px solid red";
      btn.title = `Class: ${btn.className} | ID: ${btn.id} | Aria: ${btn.getAttribute("aria-label")}`;
    });
    console.log(
      `🔴 ${buttons.length} botones resaltados. Pasa el mouse sobre ellos para ver selectores.`,
    );
  },

  // Intenta encontrar el botón "Clear queue" (suele estar escondido)
  findHiddenControls: () => {
    // A veces están en menús contextuales (ytmusic-menu-renderer)
    const menus = document.querySelectorAll("ytmusic-menu-renderer");
    console.log("Menús encontrados:", menus);
    // Aquí podrías iterar para ver si alguno tiene texto "Borrar" o "Remove"
  },
};
// Uso:
window.debugUtils = {
  YTMQueueInspector,
  YTMDataSpy,
  YTMActionSimulator,
};
