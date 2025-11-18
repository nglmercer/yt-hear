function getSongInfo() {
  try {
    const video = document.querySelector("video");
    // Si no hay video, no hay música sonando
    if (!video) return null;

    // === 1. ESTRATEGIA PRINCIPAL: API Media Session (La más fiable) ===
    // Esta API es la que usa el navegador para mostrar la info en la pantalla de bloqueo o controles multimedia
    const metadata = navigator.mediaSession?.metadata;

    // === 2. ESTRATEGIA DE RESPALDO: DOM Scraping (Barra de reproducción) ===
    const playerBar = document.querySelector("ytmusic-player-bar");

    // --- OBTENCIÓN DEL TÍTULO ---
    let title = "";

    // Intento A: Media Session
    if (metadata?.title) {
      title = metadata.title;
    }
    // Intento B: Selector DOM preciso (buscando el atributo 'title' que es texto puro)
    else if (playerBar) {
      const titleEl = playerBar.querySelector(".content-info-wrapper .title");
      // A veces el texto está en el atributo 'title', a veces en textContent
      title = titleEl?.title || titleEl?.textContent?.trim();
    }

    // Intento C: Fallback al document.title (Limpiado correctamente)
    if (!title) {
      const docTitle = document.title; // Ej: "Canción - Artista - YouTube Music"
      // Quitamos el sufijo y separamos por el guión si existe para no incluir al artista
      const cleanTitle = docTitle.replace(/ - YouTube Music$/i, "");
      // Si tiene formato "Canción - Artista", intentamos coger solo la primera parte
      // Ojo: esto es arriesgado si la canción lleva guión, pero es el último recurso.
      const parts = cleanTitle.split(" - ");
      title = parts.length > 1 ? parts[0] : cleanTitle;
    }

    // --- OBTENCIÓN DEL ARTISTA ---
    let artist = "";

    if (metadata?.artist) {
      artist = metadata.artist;
    } else if (playerBar) {
      // El artista suele estar en la clase 'subtitle' o 'byline'
      const subtitleEl =
        playerBar.querySelector(".content-info-wrapper .subtitle") ||
        playerBar.querySelector(".byline");

      if (subtitleEl) {
        // El subtítulo suele ser: "Artista • Álbum • Año"
        // Obtenemos todo el texto y separamos por el punto medio o bullets
        const text = subtitleEl.textContent || "";
        const parts = text.split(/ • | \u2022 /); // Separa por el punto gordo
        artist = parts[0]?.trim(); // La primera parte suele ser el artista
      }
    }

    if (!artist) artist = "Artista desconocido";

    // --- OBTENCIÓN DEL ÁLBUM ---
    let album = metadata?.album || null;

    if (!album && playerBar) {
      const subtitleEl = playerBar.querySelector(
        ".content-info-wrapper .subtitle",
      );
      if (subtitleEl) {
        const text = subtitleEl.textContent || "";
        const parts = text.split(/ • | \u2022 /);
        // Si hay 3 partes (Artista • Álbum • Año), el álbum es el segundo
        // Si hay 2 partes (Artista • Álbum), el álbum es el segundo
        if (parts.length >= 2) {
          album = parts[1]?.trim();
        }
      }
    }

    // --- OBTENCIÓN DE LA IMAGEN (Alta Calidad) ---
    let imageSrc = "";

    // Intentamos sacar la imagen de mayor calidad de los metadatos primero
    if (metadata?.artwork?.length > 0) {
      // Buscamos la imagen más grande disponible en el array de artwork
      const bigArt = [...metadata.artwork].sort((a, b) => {
        const sizeA = parseInt(a.sizes?.split("x")[0] || "0");
        const sizeB = parseInt(b.sizes?.split("x")[0] || "0");
        return sizeB - sizeA;
      })[0];
      imageSrc = bigArt.src;
    }

    // Si falla, buscamos en el DOM
    if (!imageSrc && playerBar) {
      const img =
        playerBar.querySelector(".thumbnail img") ||
        playerBar.querySelector("img");
      imageSrc = img?.src || "";
    }

    // Truco para forzar máxima resolución en URLs de Google (lh3.googleusercontent...)
    if (imageSrc) {
      imageSrc = imageSrc
        .replace(/w\d+-h\d+/, "w1200-h1200") // Forzar tamaño
        .replace(/=s\d+.*$/, "=s1200"); // Otra variante de tamaño
    }

    // --- DATOS TÉCNICOS (Video Element) ---
    const isPaused = video.paused;
    const elapsedSeconds = Math.floor(video.currentTime || 0);
    const songDuration = Math.floor(video.duration || 0);

    // Limpieza de URL
    const url = location.href.split("&")[0];

    return {
      title: title || "Sin título",
      artist,
      album,
      imageSrc,
      isPaused,
      elapsedSeconds,
      songDuration,
      url,
      timestamp: Date.now(),
    };
  } catch (err) {
    console.error("[YT Music] Error extrayendo info:", err);
    return null;
  }
}

window.YTM = window.YTM || {};
window.YTM.Info = {
    get: getSongInfo // Ahora se llama como window.YTM.Info.get()
};