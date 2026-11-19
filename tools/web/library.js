// The page's side of the web build's window and display (`src/platform/web.rs`): Emscripten
// library functions that run on the page's thread (the game runs on a worker, which has no
// `document` or `screen`). Linked by `tools/web/build.sh` with `--js-library`.
mergeInto(LibraryManager.library, {
  // The sizes the game offers as resolutions, in device pixels, written as eight 32-bit
  // numbers at `out`: the page's window (width, height), the screen (width, height), and the
  // two of the window in CSS pixels.
  rc2k_web_sizes__proxy: 'sync',
  rc2k_web_sizes__sig: 'vp',
  rc2k_web_sizes: (out) => {
    const dpr = window.devicePixelRatio || 1;
    const sizes = [
      Math.round(window.innerWidth * dpr), Math.round(window.innerHeight * dpr),
      Math.round(screen.width * dpr), Math.round(screen.height * dpr),
      window.innerWidth, window.innerHeight,
    ];
    sizes.forEach((v, i) => { HEAP32[(out >> 2) + i] = v; });
  },

  // The display's refreshes a second as the page measured them (`index.html`), 0 if not yet.
  rc2k_web_refresh__proxy: 'sync',
  rc2k_web_refresh__sig: 'd',
  rc2k_web_refresh: () => window.rc2kRefresh || 0,

  // The whole page (both canvases) put into the browser's full screen, or back; 1 when the
  // request went through. Only a key or click lets the browser do it: a request without one
  // answers 0 and the page stays as it was.
  rc2k_web_set_full_screen__proxy: 'sync',
  rc2k_web_set_full_screen__sig: 'ii',
  rc2k_web_set_full_screen: (on) => {
    const root = document.documentElement;
    try {
      if (on) {
        if (document.fullscreenElement) return 1;
        if (navigator.userActivation && !navigator.userActivation.isActive) return 0;
        root.requestFullscreen({ navigationUI: 'hide' }).catch(() => {});
        return 1;
      }
      if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
      return 1;
    } catch (e) {
      return 0;
    }
  },

  // Whether the page is in the browser's full screen now (it leaves by Esc or F11 on its own).
  rc2k_web_is_full_screen__proxy: 'sync',
  rc2k_web_is_full_screen__sig: 'i',
  rc2k_web_is_full_screen: () => (document.fullscreenElement ? 1 : 0),
});
