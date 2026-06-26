#!/usr/bin/env bash
set -euo pipefail

# Local-only HTTPS reverse proxy for testing browser secure-context behavior on a
# phone without routing the scrub workflow through Cloudflare/ngrok/etc.
#
# This is for synthetic demo traffic. It does not make the project HIPAA
# compliant. To avoid browser certificate warnings on iPhone, install/trust the
# generated CA certificate on the phone.

cd "$(dirname "$0")/.."

UPSTREAM="${AIRPLANE_HTTPS_UPSTREAM:-http://127.0.0.1:8099}"
HOST="${AIRPLANE_HTTPS_HOST:-0.0.0.0}"
PORT="${AIRPLANE_HTTPS_PORT:-8443}"
CERT_DIR="${AIRPLANE_CERT_DIR:-.airplane/certs}"
CA_KEY="$CERT_DIR/airplane-local-ca.key"
CA_CERT="$CERT_DIR/airplane-local-ca.pem"
SERVER_KEY="$CERT_DIR/airplane-local-server.key"
SERVER_CSR="$CERT_DIR/airplane-local-server.csr"
SERVER_CERT="$CERT_DIR/airplane-local-server.pem"
SAN_FILE="$CERT_DIR/san.conf"

mkdir -p "$CERT_DIR"

local_ips() {
  {
    ipconfig getifaddr en0 2>/dev/null || true
    ipconfig getifaddr en1 2>/dev/null || true
    ifconfig bridge100 2>/dev/null | awk '/inet / {print $2}' || true
  } | awk 'NF && $1 != "127.0.0.1"' | sort -u
}

if [[ ! -f "$CA_KEY" || ! -f "$CA_CERT" ]]; then
  openssl req -x509 -newkey rsa:2048 -days 825 -nodes \
    -keyout "$CA_KEY" -out "$CA_CERT" \
    -subj "/CN=Airplane Mode Local Dev CA" >/dev/null 2>&1
fi

{
  echo "[req]"
  echo "distinguished_name=req_distinguished_name"
  echo "req_extensions=v3_req"
  echo "prompt=no"
  echo "[req_distinguished_name]"
  echo "CN=airplane.local"
  echo "[v3_req]"
  echo "keyUsage=keyEncipherment,dataEncipherment"
  echo "extendedKeyUsage=serverAuth"
  echo "subjectAltName=@alt_names"
  echo "[alt_names]"
  echo "DNS.1=localhost"
  echo "DNS.2=airplane.local"
  echo "IP.1=127.0.0.1"
  n=2
  for ip in $(local_ips); do
    echo "IP.$n=$ip"
    n=$((n+1))
  done
} > "$SAN_FILE"

openssl req -new -newkey rsa:2048 -nodes \
  -keyout "$SERVER_KEY" -out "$SERVER_CSR" \
  -config "$SAN_FILE" >/dev/null 2>&1
openssl x509 -req -days 825 \
  -in "$SERVER_CSR" -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial \
  -out "$SERVER_CERT" -extensions v3_req -extfile "$SAN_FILE" >/dev/null 2>&1

echo "Airplane Mode local HTTPS proxy"
echo "  upstream: $UPSTREAM"
echo "  local:    https://localhost:$PORT"
for ip in $(local_ips); do
  echo "  phone:    https://$ip:$PORT"
done
echo
echo "Trust this CA on the iPhone before expecting WebGPU secure-context behavior:"
echo "  $CA_CERT"
echo
echo "Do not use this for real PHI. Synthetic demo traffic only."

python3 - "$HOST" "$PORT" "$UPSTREAM" "$SERVER_CERT" "$SERVER_KEY" <<'PY'
import http.client
import json
import ssl
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

host, port, upstream, cert, key = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5]
up = urlsplit(upstream)

class Proxy(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        sys.stderr.write("https-proxy: " + (fmt % args) + "\n")

    def do_HEAD(self):
        self._proxy()

    def do_GET(self):
        self._proxy()

    def do_POST(self):
        self._proxy()

    def _proxy(self):
        length = int(self.headers.get("Content-Length", "0") or "0")
        body = self.rfile.read(length) if length else None
        path = self.path
        headers = {k: v for k, v in self.headers.items() if k.lower() not in {"host", "connection", "content-length"}}
        if body is not None:
            headers["Content-Length"] = str(len(body))
        conn = http.client.HTTPConnection(up.hostname, up.port or 80, timeout=180)
        try:
            conn.request(self.command, path, body=body, headers=headers)
            resp = conn.getresponse()
            data = resp.read()
        except Exception as exc:
            payload = json.dumps({"ok": False, "error": f"local https proxy upstream failed: {exc}"}).encode()
            self.send_response(502)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return
        self.send_response(resp.status)
        for k, v in resp.getheaders():
            if k.lower() in {"connection", "transfer-encoding", "content-encoding", "content-length"}:
                continue
            self.send_header(k, v)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        if self.command != "HEAD":
            try:
                self.wfile.write(data)
            except BrokenPipeError:
                self.log_message("client disconnected before response body finished: %s", path)

httpd = ThreadingHTTPServer((host, port), Proxy)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(certfile=cert, keyfile=key)
httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)
httpd.serve_forever()
PY
