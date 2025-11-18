window.Logger = {
  prefix: '%c[PearObserver]',
  style: 'background: #bada55; color: #222; padding: 2px 5px; border-radius: 3px; font-weight: bold;',
  
  info: (msg, ...args) => console.log(window.Logger.prefix, window.Logger.style, msg, ...args),
  warn: (msg, ...args) => console.warn(window.Logger.prefix, window.Logger.style, msg, ...args),
  error: (msg, ...args) => console.error(window.Logger.prefix, window.Logger.style, msg, ...args),
  
  debugEnabled: true, 
  debug: (msg, ...args) => {
    if (window.Logger.debugEnabled) {
      console.debug('%c[PearDebug]', 'color: #888', msg, ...args);
    }
  }
};