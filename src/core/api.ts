// API simplificada para Tatar con adblocker y control de reproducción

export interface SongMetadata {
  id: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  thumbnail: string;
  isLiked: boolean;
  isExplicit: boolean;
}

export interface PlaybackState {
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  isMuted: boolean;
  repeatMode: 'none' | 'one' | 'all';
  shuffleMode: boolean;
}

// API principal para interactuar con YouTube Music
export class TatarAPI {
  private selectors = {
    playButton: 'button[title="Play"], button[aria-label*="Play"]',
    pauseButton: 'button[title="Pause"], button[aria-label*="Pause"]',
    nextButton: 'button[title="Next"], button[aria-label*="Next"]',
    prevButton: 'button[title="Previous"], button[aria-label*="Previous"]',
    volumeSlider: 'input[type="range"][aria-label*="Volume"], .volume-slider',
    progressBar: '#progress-bar, .progress-bar',
    songTitle: '.ytmusic-player-bar .title, .title',
    artistName: '.ytmusic-player-bar .byline, .byline',
    thumbnail: '.ytmusic-player-bar .thumbnail img, .thumbnail img',
    likeButton: 'button[title*="Like"], button[aria-label*="Like"]'
  };

  constructor() {
    this.initAdBlocker();
  }

  // Inicializar adblocker básico
  private initAdBlocker() {
    // Lista de selectores de elementos de publicidad en YouTube Music
    const adSelectors = [
      '.ytmusic-player-bar.advertisement',
      '.advertisement',
      '[data-ad-type]',
      '.ytmusic-promo',
      '.promo-container',
      'ytmusic-item-thumbnail-overlay-renderer',
      '.ytmusic-nav-bar .button-renderer[aria-label*="Premium"]'
    ];

    // Observer para eliminar publicidad dinámicamente
    const observer = new MutationObserver(() => {
      adSelectors.forEach(selector => {
        const ads = document.querySelectorAll(selector);
        ads.forEach(ad => {
          if (ad && ad.parentNode) {
            ad.parentNode.removeChild(ad);
            console.log('🚫 Ad blocked:', selector);
          }
        });
      });
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    // Eliminar publicidad existente
    setTimeout(() => {
      adSelectors.forEach(selector => {
        const ads = document.querySelectorAll(selector);
        ads.forEach(ad => {
          if (ad && ad.parentNode) {
            ad.parentNode.removeChild(ad);
          }
        });
      });
    }, 1000);
  }

  // Control de reproducción
  async play(): Promise<void> {
    const playButton = document.querySelector(this.selectors.playButton) as HTMLElement;
    if (playButton && playButton.offsetParent !== null) {
      playButton.click();
    }
  }

  async pause(): Promise<void> {
    const pauseButton = document.querySelector(this.selectors.pauseButton) as HTMLElement;
    if (pauseButton && pauseButton.offsetParent !== null) {
      pauseButton.click();
    }
  }

  async togglePlay(): Promise<boolean> {
    const playButton = document.querySelector(this.selectors.playButton) as HTMLElement;
    const pauseButton = document.querySelector(this.selectors.pauseButton) as HTMLElement;
    
    if (playButton && playButton.offsetParent !== null) {
      await this.play();
      return true;
    } else if (pauseButton && pauseButton.offsetParent !== null) {
      await this.pause();
      return false;
    }
    
    return null as any;
  }

  async next(): Promise<void> {
    const nextButton = document.querySelector(this.selectors.nextButton) as HTMLElement;
    if (nextButton) {
      nextButton.click();
    }
  }

  async previous(): Promise<void> {
    const prevButton = document.querySelector(this.selectors.prevButton) as HTMLElement;
    if (prevButton) {
      prevButton.click();
    }
  }

  async stop(): Promise<void> {
    await this.pause();
  }

  async seekTo(position: number): Promise<void> {
    const progressBar = document.querySelector(this.selectors.progressBar) as HTMLElement;
    if (progressBar) {
      const rect = progressBar.getBoundingClientRect();
      const x = rect.left + (rect.width * position / 100);
      const clickEvent = new MouseEvent('click', {
        bubbles: true,
        clientX: x,
        clientY: rect.top + rect.height / 2
      });
      progressBar.dispatchEvent(clickEvent);
    }
  }

  // Control de volumen
  async setVolume(level: number): Promise<void> {
    const volumeSlider = document.querySelector(this.selectors.volumeSlider) as HTMLInputElement;
    if (volumeSlider) {
      volumeSlider.value = level.toString();
      volumeSlider.dispatchEvent(new Event('input', { bubbles: true }));
      volumeSlider.dispatchEvent(new Event('change', { bubbles: true }));
    }
  }

  async getVolume(): Promise<number> {
    const volumeSlider = document.querySelector(this.selectors.volumeSlider) as HTMLInputElement;
    return volumeSlider ? parseInt(volumeSlider.value) : 0;
  }

  async mute(): Promise<void> {
    const volumeSlider = document.querySelector(this.selectors.volumeSlider) as HTMLInputElement;
    if (volumeSlider) {
      const currentVolume = parseInt(volumeSlider.value);
      (volumeSlider as any).dataset.previousVolume = currentVolume.toString();
      await this.setVolume(0);
    }
  }

  async unmute(): Promise<void> {
    const volumeSlider = document.querySelector(this.selectors.volumeSlider) as HTMLInputElement;
    if (volumeSlider && (volumeSlider as any).dataset.previousVolume) {
      const previousVolume = parseInt((volumeSlider as any).dataset.previousVolume);
      await this.setVolume(previousVolume);
    } else {
      await this.setVolume(50);
    }
  }

  async isMuted(): Promise<boolean> {
    const volume = await this.getVolume();
    return volume === 0;
  }

  // Información de canción
  async getCurrentSong(): Promise<SongMetadata | null> {
    const titleElement = document.querySelector(this.selectors.songTitle);
    const artistElement = document.querySelector(this.selectors.artistName);
    const thumbnailElement = document.querySelector(this.selectors.thumbnail) as HTMLImageElement;
    
    const title = titleElement?.textContent?.trim();
    const artist = artistElement?.textContent?.trim();
    const thumbnail = thumbnailElement?.src || '';
    
    if (title && artist) {
      return {
        id: this.generateSongId(title, artist),
        title,
        artist,
        album: '', // YouTube Music no muestra fácilmente el álbum actual
        duration: await this.getDuration(),
        thumbnail,
        isLiked: await this.isLiked(),
        isExplicit: false // No hay forma fácil de detectar esto
      };
    }
    
    return null;
  }

  async getPlaybackState(): Promise<PlaybackState> {
    const isPlaying = !(document.querySelector(this.selectors.playButton) as HTMLElement)?.offsetParent;
    const currentTime = await this.getCurrentTime();
    const duration = await this.getDuration();
    const volume = await this.getVolume();
    
    return {
      isPlaying,
      currentTime,
      duration,
      volume,
      isMuted: await this.isMuted(),
      repeatMode: 'none', // TODO: Implementar detección de modo repeat
      shuffleMode: false // TODO: Implementar detección de modo shuffle
    };
  }

  async getProgress(): Promise<{ current: number; total: number }> {
    return {
      current: await this.getCurrentTime(),
      total: await this.getDuration()
    };
  }

  // Métodos privados
  private async getCurrentTime(): Promise<number> {
    const progressBar = document.querySelector(this.selectors.progressBar) as HTMLElement;
    if (progressBar) {
      const timeElement = document.querySelector('.time-info, .current-time');
      if (timeElement) {
        const timeText = timeElement.textContent;
        return this.parseTime(timeText || '0:00');
      }
    }
    return 0;
  }

  private async getDuration(): Promise<number> {
    const progressBar = document.querySelector(this.selectors.progressBar) as HTMLElement;
    if (progressBar) {
      const durationElement = document.querySelector('.duration-info, .total-time');
      if (durationElement) {
        const timeText = durationElement.textContent;
        return this.parseTime(timeText || '0:00');
      }
    }
    return 0;
  }

  private async isLiked(): Promise<boolean> {
    const likeButton = document.querySelector(this.selectors.likeButton) as HTMLElement;
    if (likeButton) {
      return likeButton.getAttribute('aria-pressed') === 'true' ||
             likeButton.classList.contains('liked') ||
             likeButton.querySelector('.liked') !== null;
    }
    return false;
  }

  private generateSongId(title: string, artist: string): string {
    return btoa(`${title}-${artist}`).replace(/[^a-zA-Z0-9]/g, '').substring(0, 12);
  }

  private parseTime(timeText: string): number {
    const parts = timeText.split(':').map(p => parseInt(p.trim()));
    if (parts.length === 2) {
      return parts[0] * 60 + parts[1];
    } else if (parts.length === 3) {
      return parts[0] * 3600 + parts[1] * 60 + parts[2];
    }
    return 0;
  }
}

// Exponer la API globalmente
let tatarAPI: TatarAPI | null = null;

export function initializeTatarAPI() {
  if (!tatarAPI) {
    tatarAPI = new TatarAPI();
    (window as any).tatarAPI = tatarAPI;
    console.log('🎵 Tatar API initialized with adblocker');
  }
  return tatarAPI;
}

export function getTatarAPI(): TatarAPI | null {
  return tatarAPI || (window as any).tatarAPI;
}