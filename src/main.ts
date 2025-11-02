// Punto de entrada principal simplificado para Tatar
import { initializeTatarAPI } from './core/api.js';

// Función para esperar a que YouTube Music esté completamente cargado
function waitForYouTubeMusic(callback: () => void) {
  const ytmusicSelectors = [
    'ytmusic-app',
    'ytmusic-nav-bar',
    'ytmusic-player-bar',
    '[role="main"]',
    'main',
    '.ytmusic-app'
  ];
  
  let isLoaded = false;
  for (const selector of ytmusicSelectors) {
    if (document.querySelector(selector)) {
      isLoaded = true;
      break;
    }
  }
  
  if (isLoaded) {
    callback();
  } else {
    setTimeout(() => waitForYouTubeMusic(callback), 1000);
  }
}

// Función principal de inicialización
function initializeTatar() {
  console.log('🚀 Initializing Tatar Core...');
  
  try {
    // Inicializar la API principal
    const api = initializeTatarAPI();
    
    if (api) {
      console.log('✅ Tatar Core initialized successfully');
      console.log('🎵 AdBlocker and playback controls are active');
      
      // Exponer funciones globales para depuración
      (window as any).tatarDebug = {
        api: api,
        version: '1.0.0-simplified',
        status: () => {
          return {
            initialized: true,
            features: ['adblocker', 'playback-controls'],
            api: 'available'
          };
        }
      };
      
      console.log('🔧 Tatar Debug API available at window.tatarDebug');
    } else {
      console.error('❌ Failed to initialize Tatar API');
    }
  } catch (error) {
    console.error('❌ Error initializing Tatar:', error);
  }
}

// Inicializar cuando el DOM esté listo
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    waitForYouTubeMusic(initializeTatar);
  });
} else {
  waitForYouTubeMusic(initializeTatar);
}

// Exportar para uso en módulos
export { initializeTatarAPI };