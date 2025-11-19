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
    YtMusicSearch,

    // Debug
    YtDebug,

    ServerControl,
}

impl ScriptId {
    pub const fn content(self) -> &'static str {
        match self {
            ScriptId::Logger => include_str!("./logger.js"),
            ScriptId::Adblock => include_str!("./adblock.js"),
            ScriptId::YtMusicInfo => include_str!("./providers/songinfo.js"),
            ScriptId::TauriBridge => include_str!("./bridge.js"),
            ScriptId::YtPlayerState => include_str!("./providers/playerstate.js"),
            ScriptId::YtQueueController => include_str!("./providers/queuecontroller.js"),
            ScriptId::YtMusicObserver => include_str!("./providers/observer.js"),
            ScriptId::YtPlayerListener => include_str!("./providers/playerListeners.js"),
            ScriptId::YtMusicController => include_str!("./providers/controller.js"),
            ScriptId::YtDebug => include_str!("./providers/debug.js"),
            ScriptId::ServerControl => include_str!("./server_control.js"),
            ScriptId::YtMusicSearch => include_str!("./providers/search.js"),
        }
    }

    pub const ALL_IN_ORDER: [ScriptId; 12] = [
        ScriptId::Logger,
        ScriptId::Adblock,
        ScriptId::TauriBridge,
        ScriptId::YtMusicInfo,
        ScriptId::YtPlayerState,
        ScriptId::YtQueueController,
        ScriptId::YtMusicObserver,
        ScriptId::YtMusicController,
        ScriptId::YtPlayerListener,
        ScriptId::YtDebug,
        ScriptId::ServerControl,
        ScriptId::YtMusicSearch,
    ];
}
