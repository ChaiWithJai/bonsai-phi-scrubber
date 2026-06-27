"""PHI Scrubber Error Discovery — review server.

Adapted from shreyashankar/error-discovery-skill (SKILL.md Phase 3).
Python stdlib only — no dependencies.

Endpoints:
  GET  /              serves the HTML review app
  GET  /api/samples   current sample set
  POST /api/samples   push new samples
  GET  /api/annotations   current annotations
  POST /api/annotations   save annotations
  GET  /api/graph     2D projection of all records
  GET  /api/patterns  failure mode taxonomy
  POST /api/patterns  update taxonomy
  GET  /api/traces    all trace records
"""

import http.server
import json
import math
import os
import random
import sys
from pathlib import Path

DATA_DIR = Path(__file__).resolve().parent / "error_discovery_data"
TRACES_FILE = Path(__file__).resolve().parent / "traces.jsonl"
HTML_FILE = Path(__file__).resolve().parent / "review_app.html"
PORT = 8347


def ensure_data_dir():
    DATA_DIR.mkdir(exist_ok=True)
    for name in ["samples", "annotations", "graph", "patterns", "suggestions"]:
        path = DATA_DIR / f"{name}.json"
        if not path.exists():
            if name == "annotations":
                path.write_text("[]")
            elif name in ("samples", "suggestions"):
                path.write_text("[]")
            else:
                path.write_text("{}")


def load_traces():
    records = []
    with open(TRACES_FILE) as f:
        for line in f:
            line = line.strip()
            if line:
                records.append(json.loads(line))
    return records


def compute_graph(traces):
    """Simple 2D projection using entity-type features + PCA-like approach."""
    entity_types = sorted(set(
        e for t in traces for e in t.get("entity_types", [])
    ))
    if not entity_types:
        return {"nodes": [], "entity_types": entity_types}

    vectors = []
    for t in traces:
        vec = []
        for et in entity_types:
            vec.append(1.0 if et in t.get("entity_types", []) else 0.0)
        vec.append(t.get("hard_count", 0) / max(t.get("total_labels", 1), 1))
        vec.append(min(t.get("note_length", 0) / 600.0, 1.0))
        vectors.append(vec)

    dim = len(vectors[0])
    mean = [sum(v[d] for v in vectors) / len(vectors) for d in range(dim)]
    centered = [[v[d] - mean[d] for d in range(dim)] for v in vectors]

    def dot(a, b):
        return sum(x * y for x, y in zip(a, b))

    def norm(a):
        return math.sqrt(dot(a, a)) or 1.0

    pc1 = centered[0] if centered else [0] * dim
    for _ in range(10):
        new_pc = [0.0] * dim
        for v in centered:
            proj = dot(v, pc1)
            for d in range(dim):
                new_pc[d] += proj * v[d]
        n = norm(new_pc)
        pc1 = [x / n for x in new_pc]

    residuals = []
    for v in centered:
        proj = dot(v, pc1)
        residuals.append([v[d] - proj * pc1[d] for d in range(dim)])

    pc2 = residuals[0] if residuals else [0] * dim
    for _ in range(10):
        new_pc = [0.0] * dim
        for v in residuals:
            proj = dot(v, pc2)
            for d in range(dim):
                new_pc[d] += proj * v[d]
        n = norm(new_pc)
        pc2 = [x / n for x in new_pc]

    nodes = []
    for i, t in enumerate(traces):
        x = dot(centered[i], pc1)
        y = dot(centered[i], pc2)
        nodes.append({
            "id": t["id"],
            "x": x,
            "y": y,
            "entity_types": t.get("entity_types", []),
            "total_labels": t.get("total_labels", 0),
            "hard_count": t.get("hard_count", 0),
            "clean": t.get("clean", False),
        })
    return {"nodes": nodes, "entity_types": entity_types}


def select_initial_samples(traces):
    """Select diverse initial samples (all 21 for our small dataset)."""
    return [t["id"] for t in traces if not t.get("clean", False)]


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def _json_response(self, data, status=200):
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def _read_body(self):
        length = int(self.headers.get("Content-Length", 0))
        return self.rfile.read(length)

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def do_GET(self):
        if self.path == "/":
            html = HTML_FILE.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(html)))
            self.end_headers()
            self.wfile.write(html)

        elif self.path == "/api/traces":
            self._json_response(load_traces())

        elif self.path == "/api/samples":
            data = json.loads((DATA_DIR / "samples.json").read_text())
            self._json_response(data)

        elif self.path == "/api/annotations":
            data = json.loads((DATA_DIR / "annotations.json").read_text())
            self._json_response(data)

        elif self.path == "/api/graph":
            cached = DATA_DIR / "graph.json"
            data = json.loads(cached.read_text())
            if not data:
                traces = load_traces()
                data = compute_graph(traces)
                cached.write_text(json.dumps(data))
            self._json_response(data)

        elif self.path == "/api/patterns":
            data = json.loads((DATA_DIR / "patterns.json").read_text())
            self._json_response(data)

        elif self.path == "/api/suggestions":
            data = json.loads((DATA_DIR / "suggestions.json").read_text())
            self._json_response(data)

        else:
            self.send_error(404)

    def do_POST(self):
        body = self._read_body()

        if self.path == "/api/annotations":
            (DATA_DIR / "annotations.json").write_text(body.decode())
            self._json_response({"ok": True})

        elif self.path == "/api/samples":
            (DATA_DIR / "samples.json").write_text(body.decode())
            self._json_response({"ok": True})

        elif self.path == "/api/patterns":
            (DATA_DIR / "patterns.json").write_text(body.decode())
            self._json_response({"ok": True})

        elif self.path == "/api/suggestions":
            (DATA_DIR / "suggestions.json").write_text(body.decode())
            self._json_response({"ok": True})

        else:
            self.send_error(404)


def main():
    ensure_data_dir()

    traces = load_traces()
    graph = compute_graph(traces)
    (DATA_DIR / "graph.json").write_text(json.dumps(graph))

    samples = select_initial_samples(traces)
    (DATA_DIR / "samples.json").write_text(json.dumps(samples))

    print(f"PHI Scrubber Error Discovery")
    print(f"  Traces: {len(traces)} notes")
    print(f"  Samples: {len(samples)} (non-clean notes)")
    print(f"  Server: http://localhost:{PORT}")
    print(f"  Data dir: {DATA_DIR}")

    server = http.server.HTTPServer(("0.0.0.0", PORT), Handler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.")
        server.shutdown()


if __name__ == "__main__":
    main()
