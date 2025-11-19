// ==========================================
// 1. UTILIDADES (Para no repetir código)
// ==========================================
const YtmUtils = {
  /**
   * Limpia y extrae texto de un elemento DOM.
   */
  getText: (element, selector) => {
    if (!element) return "";
    const target = selector ? element.querySelector(selector) : element;
    return target ? (target.title || target.textContent || "").trim() : "";
  },

  /**
   * Obtiene la URL de imagen de mayor calidad posible.
   */
  getHighResImage: (imgElementOrUrl) => {
    let src = "";
    if (typeof imgElementOrUrl === "string") {
      src = imgElementOrUrl;
    } else if (imgElementOrUrl?.src) {
      src = imgElementOrUrl.src;
    }

    if (!src) return "";
    
    // Truco para forzar alta resolución en servidores de Google
    return src
      .replace(/w\d+-h\d+/, "w1200-h1200") 
      .replace(/=s\d+.*$/, "=s1200");
  },

  /**
   * Extrae el ID del video/playlist de un link (href).
   */
  extractId: (url) => {
    if (!url) return null;
    const urlObj = new URL(url, window.location.origin);
    return urlObj.searchParams.get("v") || urlObj.searchParams.get("list");
  },

  /**
   * Parsea la linea de subtitulo típica: "Artista • Álbum • 3:45"
   */
  parseSubtitle: (text) => {
    if (!text) return { artist: "", album: "", duration: "" };
    const parts = text.split(/ • | \u2022 /); // Separa por bullet point
    
    // Lógica heurística (puede variar según el contexto)
    // Caso común en lista: [Artista, Álbum, Duración]
    // Caso común en Inicio: [Tipo, Artista] o [Artista]
    
    return {
      parts, // Devolvemos el array crudo por si acaso
      artist: parts[0] || "",
      album: parts.length > 2 ? parts[1] : "", // Si hay 3 partes, el medio es album
      duration: parts.length > 1 ? parts[parts.length - 1] : "" // El último suele ser tiempo o año
    };
  }
};

// ==========================================
// 2. LÓGICA DEL REPRODUCTOR (Player actual)
// ==========================================
function getCurrentSong() {
  try {
    const video = document.querySelector("video");
    if (!video) return null;

    const metadata = navigator.mediaSession?.metadata;
    const playerBar = document.querySelector("ytmusic-player-bar");

    // --- Título ---
    let title = metadata?.title || YtmUtils.getText(playerBar, ".content-info-wrapper .title");
    if (!title) title = document.title.replace(/ - YouTube Music$/i, "");

    // --- Artista y Álbum ---
    let artist = metadata?.artist || "";
    let album = metadata?.album || "";

    if ((!artist || !album) && playerBar) {
      const subtitleText = YtmUtils.getText(playerBar, ".content-info-wrapper .subtitle");
      const parsed = YtmUtils.parseSubtitle(subtitleText);
      if (!artist) artist = parsed.artist;
      if (!album) album = parsed.album; // A veces falla si solo hay 2 elementos, pero es aceptable
    }

    // --- Imagen ---
    let imageSrc = "";
    if (metadata?.artwork?.length) {
        // Obtener la más grande
        const bigArt = [...metadata.artwork].sort((a, b) => {
            const wA = parseInt(a.sizes?.split("x")[0] || "0");
            const wB = parseInt(b.sizes?.split("x")[0] || "0");
            return wB - wA;
        })[0];
        imageSrc = bigArt.src;
    } else {
        imageSrc = playerBar?.querySelector(".thumbnail img")?.src;
    }

    return {
      type: 'current',
      title,
      artist,
      album,
      imageSrc: YtmUtils.getHighResImage(imageSrc),
      isPaused: video.paused,
      currentTime: Math.floor(video.currentTime || 0),
      duration: Math.floor(video.duration || 0),
      url: window.location.href.split("&")[0]
    };
  } catch (e) {
    console.error("Error en getCurrentSong", e);
    return null;
  }
}

// ==========================================
// 3. LÓGICA DE INICIO (Home / Cards)
// ==========================================
// Captura las tarjetas cuadradas típicas del inicio
function getHomeItems() {
  // Seleccionamos los "renderers" de items de dos filas (las tarjetas estándar)
  const items = document.querySelectorAll("ytmusic-two-row-item-renderer");
  const results = [];

  items.forEach(item => {
    const titleEl = item.querySelector(".title");
    const subtitleEl = item.querySelector(".subtitle");
    const linkEl = item.querySelector("a.yt-simple-endpoint");
    const imgEl = item.querySelector("img");

    const title = YtmUtils.getText(titleEl);
    const subtitleRaw = YtmUtils.getText(subtitleEl);
    
    // En home, el subtitulo suele ser "Sencillo • Artista" o "Álbum • Artista"
    // A veces es solo "Artista"
    
    const href = linkEl?.href || "";
    const id = YtmUtils.extractId(href);
    const imageSrc = YtmUtils.getHighResImage(imgEl);

    if (title) {
      results.push({
        type: 'home_card',
        title,
        subtitle: subtitleRaw,
        id,
        url: href,
        imageSrc
      });
    }
  });

  return results;
}

// ==========================================
// 4. LÓGICA DE BÚSQUEDA (Search / Listas)
// ==========================================
// Captura las filas horizontales (resultados de búsqueda o playlists)
function getSearchItems() {
  // Seleccionamos los items de lista responsiva
  const items = document.querySelectorAll("ytmusic-responsive-list-item-renderer");
  const results = [];

  items.forEach(item => {
    // El título está en el primer bloque de texto flexible
    const titleEl = item.querySelector(".title-column .title, .title");
    
    // La info secundaria (Artista, Album, Tiempo) está en las columnas secundarias
    // ytmusic-responsive-list-item-renderer estructura sus columnas de forma compleja
    const secondaryCols = item.querySelectorAll(".secondary-flex-columns yt-formatted-string");
    
    let artist = "";
    let album = "";
    let duration = "";

    // Intentamos mapear las columnas secundarias
    // Usualmente: [0] = Artista/Tipo, [1] = Album, [2] = Duración (puede variar)
    if (secondaryCols.length > 0) artist = YtmUtils.getText(secondaryCols[0]);
    if (secondaryCols.length > 1) album = YtmUtils.getText(secondaryCols[1]);
    if (secondaryCols.length > 2) duration = YtmUtils.getText(secondaryCols[2]);

    const linkEl = item.querySelector("a.yt-simple-endpoint");
    const imgEl = item.querySelector("img");
    
    const title = YtmUtils.getText(titleEl);
    const href = linkEl?.href || "";

    if (title && title !== "Aleatorio") { // Filtramos botones de acción
      results.push({
        type: 'search_result',
        title,
        artist,
        album,
        duration,
        id: YtmUtils.extractId(href),
        url: href,
        imageSrc: YtmUtils.getHighResImage(imgEl)
      });
    }
  });

  return results;
}

// ==========================================
// 5. EXPOSICIÓN GLOBAL
// ==========================================
window.YTM = {
    Info: {
        get: getCurrentSong,
        getRecommendations: getHomeItems,
        getResults: getSearchItems
    },
    // Método maestro para obtener todo lo visible
    getAllVisible: () => {
        return {
            current: getCurrentSong(),
            homeCards: getHomeItems(),
            listItems: getSearchItems()
        }
    }
};