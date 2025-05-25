import os
from http.server import HTTPServer, SimpleHTTPRequestHandler

os.chdir("build/web")

class COOPHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

if __name__ == "__main__":
    HTTPServer(("0.0.0.0", 8000), COOPHandler).serve_forever()
