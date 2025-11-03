(function() {  
    'use strict';  
      
    if (window.__ADBLOCK_INJECTED__) {  
        console.log('🛡️ AdBlock: Script already injected, skipping');  
        return;  
    }  
    window.__ADBLOCK_INJECTED__ = true;  
    console.log('🛡️ AdBlock: Starting injection into', window.location.href);  
      
    const CONFIG = {
        CACHE_TTL: 86400000,
        MAX_CACHE_SIZE: 1000,
        CACHE_CLEANUP_INTERVAL: 60000,
        DYNAMIC_SCAN_INTERVAL: 2000,
        DYNAMIC_SCAN_DEBOUNCE: 500,
        INIT_MAX_ATTEMPTS: 100,
        INIT_RETRY_DELAY: 50,
        ENGINE_MAX_ATTEMPTS: 50,
        ENGINE_RETRY_DELAY: 200,
        BATCH_SIZE: 50,
        YOUTUBE_CHECK_INTERVAL: 500,
        YOUTUBE_SKIP_DELAY: 100,
    };

    const WHITELIST = [];  
    
    const YOUTUBE_AD_PATTERNS = {
        domains: [
            'doubleclick.net',
            'googleadservices.com',
            'google-analytics.com',
            'googletagmanager.com',
            'googletagservices.com',
        ],
        paths: [
            '/pagead/',
            '/pcs/click',
        ],
        params: [
            'ad_type',
            'adurl',
        ]
    };
      
    const nativeFetch = window.fetch;  
    const nativeXHR = window.XMLHttpRequest;  
    
    let invoke = null;  
    let adblockReady = false;  
    let cosmeticStyleElement = null;  
    let currentResources = null;
    
    const blockCache = new Map();
    const seenClasses = new Set();  
    const seenIds = new Set();
    const pendingChecks = new Map();
    
    let cleanupInterval = null;
    let dynamicScanInterval = null;
    let debounceTimer = null;
    let youtubeAdSkipper = null;
    
    const stats = {
        networkBlocked: 0,
        cosmeticBlocked: 0,
        youtubeAdsSkipped: 0,
        youtubeTrackingBlocked: 0
    };
      
    function isWhitelisted(url) {  
        return WHITELIST.some(domain => url.includes(domain));  
    }

    function isIpcRequest(url) {  
        return url.startsWith('tauri://') ||  
               url.startsWith('ipc://') ||  
               url.includes('__TAURI_IPC__');  
    }

    function isSafeUrl(url) {
        return !url || 
               url.startsWith('data:') ||
               url.startsWith('blob:') ||
               url.startsWith('about:') ||
               isIpcRequest(url) ||
               isWhitelisted(url);
    }
    
    function isYouTubeTracking(url) {
        if (!url) return false;
        
        try {
            const urlObj = new URL(url);
            const hostname = urlObj.hostname;
            const pathname = urlObj.pathname;
            
            for (const domain of YOUTUBE_AD_PATTERNS.domains) {
                if (hostname.includes(domain)) {
                    console.log('🛡️ AdBlock: [TRACKING] Blocked:', hostname);
                    stats.youtubeTrackingBlocked++;
                    return true;
                }
            }
            
            if (pathname.includes('/pagead/') || pathname.includes('/pcs/click')) {
                console.log('🛡️ AdBlock: [TRACKING] Blocked path:', pathname);
                stats.youtubeTrackingBlocked++;
                return true;
            }
            
            return false;
        } catch (e) {
            return false;
        }
    }

    function cleanup() {
        if (cleanupInterval) clearInterval(cleanupInterval);
        if (dynamicScanInterval) clearInterval(dynamicScanInterval);
        if (debounceTimer) clearTimeout(debounceTimer);
        if (youtubeAdSkipper) clearInterval(youtubeAdSkipper);
        blockCache.clear();
        seenClasses.clear();
        seenIds.clear();
        pendingChecks.clear();
        
        console.log('🛡️ AdBlock: Final Stats:', stats);
    }

    window.addEventListener('beforeunload', cleanup);
      
    async function init() {  
        console.log('🛡️ AdBlock: [INIT] Starting initialization...');  
          
        let attempts = 0;  
        while (!window.__TAURI__?.core?.invoke && attempts < CONFIG.INIT_MAX_ATTEMPTS) {  
            await new Promise(r => setTimeout(r, CONFIG.INIT_RETRY_DELAY));  
            attempts++;  
        }  
          
        if (!window.__TAURI__?.core?.invoke) {  
            console.error('🛡️ AdBlock: [INIT] ❌ Failed to initialize Tauri API');  
            return false;  
        }  
          
        console.log('🛡️ AdBlock: [INIT] ✅ Tauri API found');  
        invoke = window.__TAURI__.core.invoke;  
          
        attempts = 0;  
        while (!adblockReady && attempts < CONFIG.ENGINE_MAX_ATTEMPTS) {  
            try {  
                adblockReady = await invoke('is_adblock_ready');  
                if (!adblockReady) {  
                    await new Promise(r => setTimeout(r, CONFIG.ENGINE_RETRY_DELAY));  
                    attempts++;  
                }  
            } catch (e) {  
                console.error('🛡️ AdBlock: [INIT] Error checking engine:', e);  
                await new Promise(r => setTimeout(r, CONFIG.ENGINE_RETRY_DELAY));  
                attempts++;  
            }  
        }  
          
        if (!adblockReady) {  
            console.warn('🛡️ AdBlock: [INIT] ⚠️ Engine failed to initialize');  
            return false;  
        }  
          
        console.log('🛡️ AdBlock: [INIT] ✅ Engine ready!');  
        
        await applyCosmeticFilters();
        startBackgroundTasks();
        
        if (window.location.hostname.includes('youtube.com')) {
            injectYouTubeAdSkipper();
        }
          
        console.log('🛡️ AdBlock: [INIT] ✅ Complete');  
        return true;  
    }

    function injectYouTubeAdSkipper() {
        console.log('🛡️ AdBlock: [YOUTUBE] Injecting ad skipper...');
        
        const style = document.createElement('style');
        style.textContent = `
            /* Ocultar overlay de anuncios pero mantener funcionalidad */
            .video-ads.ytp-ad-module,
            .ytp-ad-player-overlay,
            .ytp-ad-player-overlay-layout,
            .ytp-ad-text-overlay,
            .ytp-ad-image-overlay {
                opacity: 0 !important;
                pointer-events: none !important;
                height: 0 !important;
                overflow: hidden !important;
            }
            
            /* Ocultar banners de anuncios */
            ytd-display-ad-renderer,
            ytd-banner-promo-renderer,
            ytd-companion-slot-renderer,
            #masthead-ad,
            .ytd-promoted-sparkles-web-renderer {
                display: none !important;
            }
            
            /* Mantener el video visible durante anuncios */
            .html5-video-container {
                opacity: 1 !important;
            }
        `;
        (document.head || document.documentElement).appendChild(style);
        
        function skipAd() {
            try {
                const skipButton = document.querySelector('.ytp-ad-skip-button, .ytp-ad-skip-button-modern, .ytp-skip-ad-button');
                if (skipButton && skipButton.offsetParent !== null) {
                    console.log('🛡️ AdBlock: [YOUTUBE] Clicking skip button');
                    skipButton.click();
                    stats.youtubeAdsSkipped++;
                    return true;
                }
                
                const video = document.querySelector('video.html5-main-video');
                if (video) {
                    const isAd = document.querySelector('.ad-showing, .ytp-ad-player-overlay');
                    if (isAd) {
                        video.playbackRate = 16;
                        
                        if (video.duration && isFinite(video.duration) && video.duration > 0) {
                            video.currentTime = video.duration - 0.1;
                            console.log('🛡️ AdBlock: [YOUTUBE] Skipped to end of ad');
                            stats.youtubeAdsSkipped++;
                            return true;
                        }
                    } else {
                        if (video.playbackRate !== 1) {
                            video.playbackRate = 1;
                        }
                    }
                }
                
                const closeButtons = document.querySelectorAll('.ytp-ad-overlay-close-button, button[aria-label*="Close ad"]');
                closeButtons.forEach(btn => {
                    if (btn.offsetParent !== null) {
                        btn.click();
                        console.log('🛡️ AdBlock: [YOUTUBE] Closed ad banner');
                    }
                });
                
            } catch (e) {
                console.error('🛡️ AdBlock: [YOUTUBE] Error skipping ad:', e);
            }
            return false;
        }
        
        youtubeAdSkipper = setInterval(() => {
            skipAd();
        }, CONFIG.YOUTUBE_CHECK_INTERVAL);
        
        document.addEventListener('yt-navigate-finish', () => {
            setTimeout(skipAd, CONFIG.YOUTUBE_SKIP_DELAY);
        });
        
        const playerObserver = new MutationObserver(() => {
            const isAd = document.querySelector('.ad-showing, .ytp-ad-player-overlay');
            if (isAd) {
                skipAd();
            }
        });
        
        const checkPlayer = setInterval(() => {
            const player = document.querySelector('.html5-video-player');
            if (player) {
                playerObserver.observe(player, {
                    attributes: true,
                    attributeFilter: ['class'],
                    childList: true,
                    subtree: true
                });
                clearInterval(checkPlayer);
                console.log('🛡️ AdBlock: [YOUTUBE] ✅ Ad skipper active');
            }
        }, 1000);
        
        setTimeout(() => clearInterval(checkPlayer), 10000);
    }

    async function applyCosmeticFilters() {  
        if (!adblockReady || !invoke) {  
            console.log('🛡️ AdBlock: [COSMETIC] Skipping - engine not ready');  
            return;  
        }  
          
        try {  
            console.log('🛡️ AdBlock: [COSMETIC] Fetching resources for', window.location.href);  
            currentResources = await invoke('get_cosmetic_resources', {  
                url: window.location.href  
            });  
              
            console.log('🛡️ AdBlock: [COSMETIC] Received:', {  
                hideSelectors: currentResources.hide_selectors?.length || 0,  
                exceptions: currentResources.exceptions?.length || 0,  
                hasScript: !!currentResources.injected_script,  
                proceduralActions: currentResources.procedural_actions?.length || 0,  
                generichide: currentResources.generichide  
            });  
              
            if (!cosmeticStyleElement) {  
                cosmeticStyleElement = document.createElement('style');  
                cosmeticStyleElement.id = '__adblock_cosmetic_filters__';  
                (document.head || document.documentElement).appendChild(cosmeticStyleElement);  
            }  
              
            if (currentResources.hide_selectors?.length > 0) {  
                const rules = currentResources.hide_selectors
                    .map(s => `${s} { display: none !important; visibility: hidden !important; }`)
                    .join('\n');
                cosmeticStyleElement.textContent = rules;
                stats.cosmeticBlocked += currentResources.hide_selectors.length;
                console.log('🛡️ AdBlock: [COSMETIC] Applied', currentResources.hide_selectors.length, 'hide selectors');  
            }  
              
            if (currentResources.procedural_actions?.length > 0) {  
                applyProceduralActions(currentResources.procedural_actions);  
            }  
              
            if (currentResources.injected_script) {  
                try {
                    const script = document.createElement('script');  
                    script.textContent = currentResources.injected_script;  
                    (document.head || document.documentElement).appendChild(script);  
                    script.remove();  
                    console.log('🛡️ AdBlock: [COSMETIC] Injected scriptlet');
                } catch (e) {
                    console.error('🛡️ AdBlock: [COSMETIC] Error injecting scriptlet:', e);
                }
            }  
        } catch (e) {  
            console.error('🛡️ AdBlock: [COSMETIC] Error:', e);  
        }  
    }  
      
    function applyProceduralActions(actions) {  
        let applied = 0;
        
        actions.forEach(actionJson => {  
            try {  
                const filter = JSON.parse(actionJson);  
                  
                if (filter.selector?.length === 1 &&   
                    filter.selector[0].CssSelector &&   
                    filter.action?.Style) {  
                    const selector = filter.selector[0].CssSelector;  
                    const style = filter.action.Style;  
                    const rule = `${selector} { ${style} }`;  
                    cosmeticStyleElement.textContent += '\n' + rule;
                    applied++;
                    return;  
                }  
                  
                if (applyComplexProceduralFilter(filter)) {
                    applied++;
                }
            } catch (e) {  
                console.error('🛡️ AdBlock: [PROCEDURAL] Error parsing:', actionJson, e);  
            }  
        });  
        
        if (applied > 0) {
            console.log('🛡️ AdBlock: [PROCEDURAL] Applied', applied, 'procedural actions');
        }
    }  
      
    function applyComplexProceduralFilter(filter) {  
        try {
            if (filter.action?.RemoveAttr) {  
                const attr = filter.action.RemoveAttr;  
                const selector = filter.selector?.[0]?.CssSelector;  
                if (selector) {  
                    document.querySelectorAll(selector).forEach(el => {  
                        el.removeAttribute(attr);  
                    });
                    return true;
                }  
            }
            
            if (filter.action?.RemoveClass) {
                const className = filter.action.RemoveClass;
                const selector = filter.selector?.[0]?.CssSelector;
                if (selector) {
                    document.querySelectorAll(selector).forEach(el => {
                        el.classList.remove(className);
                    });
                    return true;
                }
            }
        } catch (e) {
            console.error('🛡️ AdBlock: [PROCEDURAL] Error applying filter:', e);
        }
        return false;
    }  
      
    async function applyDynamicHiding() {  
        if (!adblockReady || !invoke || !currentResources || currentResources.generichide) {
            return;  
        }
          
        const newClasses = [];  
        const newIds = [];  
          
        const classElements = document.querySelectorAll('[class]');
        for (const el of classElements) {
            for (const cls of el.classList) {
                if (cls && cls.length > 2 && !seenClasses.has(cls)) {
                    seenClasses.add(cls);  
                    newClasses.push(cls);  
                }
            }
        }
          
        const idElements = document.querySelectorAll('[id]');
        for (const el of idElements) {
            if (el.id && el.id.length > 2 && !seenIds.has(el.id)) {
                seenIds.add(el.id);  
                newIds.push(el.id);  
            }
        }
          
        if (newClasses.length === 0 && newIds.length === 0) {  
            return;  
        }  
          
        try {  
            const selectors = await invoke('get_hidden_class_id_selectors', {  
                classes: newClasses,  
                ids: newIds,  
                exceptions: currentResources.exceptions || []  
            });  
              
            if (selectors?.length > 0) {  
                const style = selectors
                    .map(s => `${s} { display: none !important; visibility: hidden !important; }`)
                    .join('\n');
                cosmeticStyleElement.textContent += '\n' + style;
                stats.cosmeticBlocked += selectors.length;
                console.log('🛡️ AdBlock: [DYNAMIC] Applied', selectors.length, 'new selectors');  
            }  
        } catch (e) {  
            console.error('🛡️ AdBlock: [DYNAMIC] Error:', e);  
        }  
    }  
      
    async function checkUrl(url, type) {  
        if (!adblockReady || isSafeUrl(url)) {  
            return false;  
        }
        
        if (isYouTubeTracking(url)) {
            return true;
        }
          
        const cacheKey = `${url}|${type}`;
        
        const cached = blockCache.get(cacheKey);  
        if (cached && (Date.now() - cached.time < CONFIG.CACHE_TTL)) {  
            return cached.blocked;  
        }

        if (pendingChecks.has(cacheKey)) {
            return pendingChecks.get(cacheKey);
        }
          
        const checkPromise = (async () => {
            try {  
                const blocked = await invoke('is_url_blocked', {  
                    url,  
                    sourceUrl: window.location.href,  
                    requestType: type  
                });  
                  
                blockCache.set(cacheKey, { blocked, time: Date.now() });  
                  
                if (blockCache.size > CONFIG.MAX_CACHE_SIZE) {  
                    cleanupCache();
                }  
                  
                if (blocked) {
                    stats.networkBlocked++;
                    console.log('🛡️ AdBlock: [BLOCK]', type, url.substring(0, 100));  
                }  
                  
                return blocked;  
            } catch (e) {  
                console.error('🛡️ AdBlock: [ERROR] Failed to check URL:', e.message);  
                return false;  
            } finally {
                pendingChecks.delete(cacheKey);
            }
        })();

        pendingChecks.set(cacheKey, checkPromise);
        return checkPromise;
    }

    function cleanupCache() {
        const now = Date.now();
        const entries = Array.from(blockCache.entries());
        
        let removed = 0;
        for (const [key, value] of entries) {
            if (now - value.time > CONFIG.CACHE_TTL) {
                blockCache.delete(key);
                removed++;
            }
        }
        
        if (blockCache.size > CONFIG.MAX_CACHE_SIZE) {
            const remaining = Array.from(blockCache.entries());
            remaining.sort((a, b) => a[1].time - b[1].time);
            const toRemove = remaining.slice(0, blockCache.size - CONFIG.MAX_CACHE_SIZE);
            toRemove.forEach(([key]) => blockCache.delete(key));
            removed += toRemove.length;
        }
        
        if (removed > 0) {
            console.log(`🛡️ AdBlock: [CACHE] Cleaned ${removed} entries`);
        }
    }
      
    window.fetch = async function(...args) {  
        const url = args[0];  
        const urlStr = typeof url === 'string' ? url : (url?.url || '');  
          
        if (isSafeUrl(urlStr)) {  
            return nativeFetch.apply(this, args);  
        }  
          
        const options = args[1] || {};  
        if (options.method === 'OPTIONS' || options.method === 'HEAD') {  
            return nativeFetch.apply(this, args);  
        }  
          
        if (await checkUrl(urlStr, 'fetch')) {  
            return new Response('', { 
                status: 200,
                statusText: 'OK',
                headers: new Headers()
            });  
        }  
          
        return nativeFetch.apply(this, args);  
    };  
      
    window.XMLHttpRequest = function() {  
        const xhr = new nativeXHR();  
        const originalOpen = xhr.open;  
        const originalSend = xhr.send;  
        let requestUrl = '';  
        let requestMethod = '';
          
        xhr.open = function(method, url, ...rest) {
            requestMethod = method;
            requestUrl = url;  
            return originalOpen.apply(this, [method, url, ...rest]);  
        };  
          
        xhr.send = async function(...args) {
            if (requestUrl && 
                !isSafeUrl(requestUrl) && 
                requestMethod !== 'OPTIONS' &&
                requestMethod !== 'HEAD') {
                
                const blocked = await checkUrl(requestUrl, 'xhr');
                
                if (blocked) {
                    Object.defineProperty(xhr, 'status', { value: 200, configurable: true });
                    Object.defineProperty(xhr, 'statusText', { value: 'OK', configurable: true });
                    Object.defineProperty(xhr, 'readyState', { value: 4, configurable: true });
                    Object.defineProperty(xhr, 'responseText', { value: '', configurable: true });
                    
                    setTimeout(() => {
                        if (xhr.onreadystatechange) xhr.onreadystatechange(new Event('readystatechange'));
                        if (xhr.onload) xhr.onload(new Event('load'));
                    }, 0);
                    
                    return;
                }
            }
            
            return originalSend.apply(this, args);  
        };  
          
        return xhr;  
    };  
      
    const observer = new MutationObserver(mutations => {  
        if (!adblockReady) return;  
          
        let hasNewClassesOrIds = false;  
        const elementsToCheck = [];
          
        for (const mutation of mutations) {  
            if (mutation.type === 'attributes') {
                hasNewClassesOrIds = true;
                continue;
            }

            for (const node of mutation.addedNodes) {  
                if (node.nodeType !== 1) continue;  
                  
                if (node.classList?.length > 0 || node.id) {  
                    hasNewClassesOrIds = true;  
                }  
                  
                const tag = node.tagName?.toLowerCase();  
                  
                if (['script', 'iframe', 'img', 'link'].includes(tag)) {
                    elementsToCheck.push(node);
                }
            }  
        }

        if (elementsToCheck.length > 0) {
            checkElements(elementsToCheck);
        }
          
        if (hasNewClassesOrIds) {  
            debouncedDynamicHiding();  
        }  
    });

    async function checkElements(elements) {
        for (const node of elements) {
            const tag = node.tagName.toLowerCase();
            const src = node.src || node.href || node.data || 
                       node.getAttribute('src') || node.getAttribute('href') || 
                       node.getAttribute('data');
            
            if (!src || isSafeUrl(src)) continue;
            
            try {
                const blocked = await checkUrl(src, tag);
                if (blocked && node.parentNode) {
                    node.remove();
                }
            } catch (e) {
                console.error('🛡️ AdBlock: [OBSERVER] Error checking element:', e);
            }
        }
    }
      
    observer.observe(document.documentElement, {  
        childList: true,  
        subtree: true,  
        attributes: true,  
        attributeFilter: ['class', 'id', 'src', 'href', 'data']  
    });  
      
    function startBackgroundTasks() {
        cleanupInterval = setInterval(() => {
            cleanupCache();
        }, CONFIG.CACHE_CLEANUP_INTERVAL);

        dynamicScanInterval = setInterval(() => {  
            if (adblockReady) {  
                applyDynamicHiding();  
            }  
        }, CONFIG.DYNAMIC_SCAN_INTERVAL);
        
        setInterval(() => {
            if (stats.networkBlocked > 0 || stats.cosmeticBlocked > 0 || 
                stats.youtubeAdsSkipped > 0 || stats.youtubeTrackingBlocked > 0) {
                console.log('🛡️ AdBlock: Stats Update:', stats);
            }
        }, 30000);
    }

    function debouncedDynamicHiding() {  
        if (debounceTimer) clearTimeout(debounceTimer);  
        debounceTimer = setTimeout(() => {  
            applyDynamicHiding();  
        }, CONFIG.DYNAMIC_SCAN_DEBOUNCE);  
    }
      
    window.addEventListener('load', () => {  
        console.log('🛡️ AdBlock: [EVENT] Page loaded, re-applying cosmetic filters');  
        applyCosmeticFilters().then(() => {  
            applyDynamicHiding();  
        });  
    });

    let lastUrl = location.href;
    new MutationObserver(() => {
        const currentUrl = location.href;
        if (currentUrl !== lastUrl) {
            lastUrl = currentUrl;
            console.log('🛡️ AdBlock: [EVENT] URL changed, re-applying filters');
            
            seenClasses.clear();
            seenIds.clear();
            
            if (window.location.hostname.includes('youtube.com')) {
                if (!youtubeAdSkipper) {
                    injectYouTubeAdSkipper();
                }
            }
            
            applyCosmeticFilters().then(() => {
                applyDynamicHiding();
            });
        }
    }).observe(document, { subtree: true, childList: true });
      
    init().then(success => {  
        if (success) {  
            console.log('🛡️ AdBlock: ✅ Fully operational');  
            applyDynamicHiding();  
        } else {  
            console.warn('🛡️ AdBlock: ⚠️ Initialization incomplete');  
        }  
    });
      
    window.__ADBLOCK_INITIALIZED__ = true;  
})();