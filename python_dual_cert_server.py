#!/usr/bin/env python3
"""
Dual Certificate Support - Python Proof of Concept

This server demonstrates true dual certificate support (ECDSA + RSA) using
Python's built-in ssl module with multiple load_cert_chain() calls.

The server automatically presents the appropriate certificate based on
client cipher preferences without requiring callbacks.
"""

import http.server
import socketserver
import ssl
import sys
import json
from datetime import datetime

def check_python_version():
    """Ensure we're running on Python 3.8+ for SSL features"""
    if sys.version_info < (3, 8):
        print(f"ERROR: This script requires Python 3.8 or newer.")
        print(f"You are running Python {sys.version_info.major}.{sys.version_info.minor}.")
        sys.exit(1)

    print(f"✓ Running on Python {sys.version_info.major}.{sys.version_info.minor}")

class DualCertHandler(http.server.SimpleHTTPRequestHandler):
    """Custom handler for our dual certificate server"""

    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            self.wfile.write(b'Hello from Python dual certificate server!')

        elif self.path == '/info':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()

            info = {
                "message": "Python dual certificate server",
                "approach": "Multiple ssl.load_cert_chain() calls",
                "certificates": ["ECDSA (P-256)", "RSA (2048-bit)"],
                "automatic_selection": True,
                "timestamp": datetime.now().isoformat()
            }

            self.wfile.write(json.dumps(info, indent=2).encode())

        elif self.path == '/cert-info':
            # Get certificate information from the SSL connection
            cert = self.connection.getpeercert(binary_form=True)
            if cert:
                import cryptography.x509
                from cryptography.hazmat.backends import default_backend

                x509_cert = cryptography.x509.load_der_x509_certificate(cert, default_backend())
                public_key = x509_cert.public_key()

                if hasattr(public_key, 'curve'):
                    key_type = f"ECDSA ({public_key.curve.name})"
                elif hasattr(public_key, 'key_size'):
                    key_type = f"RSA ({public_key.key_size}-bit)"
                else:
                    key_type = "Unknown"

                self.send_response(200)
                self.send_header('Content-type', 'application/json')
                self.end_headers()

                cert_info = {
                    "certificate_type": key_type,
                    "subject": str(x509_cert.subject),
                    "issuer": str(x509_cert.issuer),
                    "serial_number": str(x509_cert.serial_number)
                }

                self.wfile.write(json.dumps(cert_info, indent=2).encode())
            else:
                self.send_error(500, "Could not retrieve certificate information")
        else:
            self.send_error(404, "Not found")

def create_dual_cert_context():
    """Create SSL context with dual certificate support"""
    print("Creating SSL context...")

    # Create TLS server context
    ssl_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)

    # Load BOTH certificate chains into the same context
    # OpenSSL will automatically select the appropriate one based on client preferences

    print("Loading ECDSA certificate...")
    try:
        ssl_ctx.load_cert_chain('certs/ecdsa-cert.pem', 'certs/ecdsa-key.pem')
        print("✓ ECDSA certificate loaded successfully")
    except Exception as e:
        print(f"✗ Failed to load ECDSA certificate: {e}")
        sys.exit(1)

    print("Loading RSA certificate...")
    try:
        ssl_ctx.load_cert_chain('certs/rsa-cert.pem', 'certs/rsa-key.pem')
        print("✓ RSA certificate loaded successfully")
    except Exception as e:
        print(f"✗ Failed to load RSA certificate: {e}")
        sys.exit(1)

    print("✓ Dual certificate SSL context created successfully")
    return ssl_ctx

def main():
    """Main server function"""
    check_python_version()

    PORT = 8443

    print(f"\n🔒 Python Dual Certificate Server")
    print(f"   Certificates: ECDSA + RSA")
    print(f"   Port: {PORT}")
    print(f"   Automatic selection based on client cipher preferences")

    # Create SSL context with dual certificates
    ssl_ctx = create_dual_cert_context()

    # Create and start server
    with socketserver.TCPServer(("", PORT), DualCertHandler) as httpd:
        httpd.socket = ssl_ctx.wrap_socket(httpd.socket, server_side=True)

        print(f"\n🚀 Server started on https://localhost:{PORT}")
        print(f"📋 Available endpoints:")
        print(f"   GET /          - Hello message")
        print(f"   GET /info      - Server information")
        print(f"   GET /cert-info - Certificate details")
        print(f"\n🧪 Test commands:")
        print(f"   # Default (should use ECDSA)")
        print(f"   curl -k https://localhost:{PORT}")
        print(f"   ")
        print(f"   # Force RSA certificate")
        print(f"   openssl s_client -connect localhost:{PORT} -cipher 'ECDHE-RSA-AES256-GCM-SHA384'")
        print(f"   ")
        print(f"   # Force ECDSA certificate")
        print(f"   openssl s_client -connect localhost:{PORT} -cipher 'ECDHE-ECDSA-AES256-GCM-SHA384'")
        print(f"\n⏹️  Press Ctrl+C to stop")

        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print(f"\n🛑 Server stopped")

if __name__ == "__main__":
    main()