class YouTubeMusicSearchController {
    constructor() {
        this.searchBox = null;
        this.inputField = null;
    }

    _init() {
        this.searchBox = document.querySelector('ytmusic-search-box');
        if (!this.searchBox) return false;
        this.inputField = this.searchBox.querySelector('input');
        return !!this.inputField;
    }

    async search(query) {
        if (!this._init()) {
            console.error("❌ Buscador no encontrado en el DOM");
            return;
        }

        // 1. Abrir el buscador si está cerrado
        if (!this.searchBox.opened) {
            const searchButton = document.querySelector('ytmusic-search-box tp-yt-paper-icon-button') 
                              || document.querySelector('ytmusic-nav-bar tp-yt-paper-icon-button[icon="yt-icons:search"]');
            if (searchButton) {
                searchButton.click();
                // Esperar a que la animación de apertura termine y el input sea visible
                await new Promise(r => setTimeout(r, 300));
            }
        }

        // 2. Preparar el input
        this.inputField.focus();
        this.inputField.value = query;

        // 3. Disparar eventos de escritura para reactivar el estado interno (React/Polymer)
        // 'composed: true' es vital para atravesar el Shadow DOM
        const inputEvent = new InputEvent('input', { bubbles: true, composed: true });
        this.inputField.dispatchEvent(inputEvent);
        
        const changeEvent = new Event('change', { bubbles: true, composed: true });
        this.inputField.dispatchEvent(changeEvent);

        // 4. Simular la tecla ENTER con fuerza bruta (KeyDown + KeyPress + KeyUp)
        const keyOptions = {
            key: 'Enter',
            code: 'Enter',
            keyCode: 13,
            which: 13,
            bubbles: true,
            cancelable: true,
            composed: true,
            view: window
        };

        this.inputField.dispatchEvent(new KeyboardEvent('keydown', keyOptions));
        this.inputField.dispatchEvent(new KeyboardEvent('keypress', keyOptions)); 
        this.inputField.dispatchEvent(new KeyboardEvent('keyup', keyOptions));

        this.inputField.blur(); 

        console.log(`🔍 Intento de búsqueda simulada: "${query}"`);
    }
}

// Inicialización
window.YTM = window.YTM || {};
window.YTM.Search = new YouTubeMusicSearchController();