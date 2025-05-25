import os
from http.server import HTTPServer, SimpleHTTPRequestHandler

os.chdir("build/web")

class COOPHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        # Required for SharedArrayBuffer in modern browsers
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

if __name__ == "__main__":
    # Serve on port 8000
    HTTPServer(("0.0.0.0", 8000), COOPHandler).serve_forever()
