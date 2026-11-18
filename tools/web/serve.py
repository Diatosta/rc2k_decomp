#!/usr/bin/env python3
"""Serves the web build's folder with the two headers its threads need (cross-origin isolation:
SharedArrayBuffer). `python3 tools/web/serve.py [folder] [port]`; on a phone, a-Shell's Python
runs it the same way, or the folder's `coi-serviceworker.js` adds the headers itself."""
import http.server, sys, os, functools

class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

Handler.extensions_map[".wasm"] = "application/wasm"
folder = sys.argv[1] if len(sys.argv) > 1 else "."
port = int(sys.argv[2]) if len(sys.argv) > 2 else 8000
server = http.server.ThreadingHTTPServer(("127.0.0.1", port), functools.partial(Handler, directory=folder))
print(f"Serving {folder} at http://127.0.0.1:{port}/ (Ctrl+C stops)")
try:
    server.serve_forever()
except KeyboardInterrupt:
    print("\nStopped.")
finally:
    server.server_close()
