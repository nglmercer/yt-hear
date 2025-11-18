// src-tauri/src/scripts/mod.rs

/// Identificador único para cada script que inyectamos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptId {
    // Core
    Logger,
    Adblock,
    TauriBridge, 
    
    // Providers 
    YtMusicInfo,
    YtPlayerState,
    YtQueueController,
    
    YtMusicObserver,
    YtPlayerListener,
    
    YtMusicController,
    
    // Debug
    YtDebug,
    ServerControl,  
}

impl ScriptId {
    pub const fn content(self) -> &'static str {
        match self {
            
            ScriptId::Logger => include_str!("./logger.js"),
            ScriptId::Adblock => include_str!("./adblock.js"),
            ScriptId::TauriBridge => include_str!("./bridge.js"),
            
            ScriptId::YtMusicInfo => include_str!("./providers/songinfo.js"),
            ScriptId::YtPlayerState => include_str!("./providers/playerstate.js"),
            ScriptId::YtQueueController => include_str!("./providers/queuecontroller.js"),
            
            ScriptId::YtMusicObserver => include_str!("./providers/observer.js"),
            ScriptId::YtPlayerListener => include_str!("./providers/playerListeners.js"),
            
            ScriptId::YtMusicController => include_str!("./providers/controller.js"),
            ScriptId::YtDebug => include_str!("./providers/debug.js"),
            ScriptId::ServerControl => include_str!("./server_control.js"),
        }
    }

    pub const ALL_IN_ORDER: [ScriptId; 11] = [
        ScriptId::Logger,
        ScriptId::Adblock,
        ScriptId::TauriBridge,      
        ScriptId::YtMusicInfo,      
        ScriptId::YtPlayerState,
        ScriptId::YtQueueController,
        ScriptId::YtMusicController,
        ScriptId::YtMusicObserver, 
        ScriptId::YtPlayerListener,
        ScriptId::YtDebug,
        ScriptId::ServerControl,
    ];
}