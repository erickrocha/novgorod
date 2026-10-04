#!/usr/bin/env python3
"""Mercado Pago test double (NOV-3 T-CHK-02). Python stdlib only; state is in memory.

kremlin reaches it through MP_API_BASE_URL. Endpoints the app uses (application/src/infrastructure/mercado_pago.rs):
  POST /v1/payments                      charge  -> payment JSON
  GET  /v1/payments/{id}                 status
  GET  /v1/payments/search?external_reference=...   -> {"results": [...]}
Control endpoints (not part of Mercado Pago):
  GET    /_mpd/calls                     every call received, oldest first (method, path, query, headers subset, body)
  DELETE /_mpd/calls                     reset everything: calls, payments, config
  GET    /_mpd/payments                  payments created so far (ids are what webhooks carry)
  GET    /_mpd/config   POST /_mpd/config
         {"mode": "approved|rejected|pending|authorized|error",   what the next charges answer (default approved)
          "amount_cents": null|int,       answer this amount instead of the one received (inconsistent-amount case)
          "currency": "BRL", "collector_id": int, "payment_type_id": "credit_card", "external_reference": null|str}
  POST   /_mpd/payments/{id}  {"status": "approved", "amount_cents": ...}   change a stored payment (status/search/webhook then see it)
A card token that starts with "reject", "pending", "authorized" or "error" overrides the mode for that charge.
A repeated X-Idempotency-Key returns the stored payment instead of creating another (as the real API does).
"""
import json, os, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

LOCK = threading.Lock()
DEFAULTS = {"mode": "approved", "amount_cents": None, "currency": "BRL", "payment_type_id": "credit_card",
            "collector_id": int(os.environ.get("MPD_COLLECTOR_ID", "424242")), "external_reference": None}
STATE = {"config": dict(DEFAULTS), "calls": [], "payments": {}, "keys": {}, "next_id": 9000001}
STATUS = {"approved": "approved", "rejected": "rejected", "pending": "pending", "authorized": "authorized"}


def payment_json(p):
    return {"id": p["id"], "status": p["status"], "transaction_amount": p["amount_cents"] / 100.0,
            "currency_id": p["currency"], "external_reference": p["external_reference"],
            "collector_id": p["collector_id"], "payment_type_id": p["payment_type_id"],
            "payment_method_id": p["payment_method_id"],
            "card": {"last_four_digits": "4242", "expiration_month": 12, "expiration_year": 2030,
                     "cardholder": {"name": "Test Holder"}}}


class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def reply(self, code, obj):
        raw = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def body(self):
        n = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(n) if n else b""
        try:
            return json.loads(raw) if raw else None
        except ValueError:
            return raw.decode(errors="replace")

    def handle_any(self, method):
        u = urlparse(self.path)
        path, q, body = u.path, parse_qs(u.query), self.body()
        with LOCK:
            if path.startswith("/_mpd/"):
                return self.control(method, path, body)
            STATE["calls"].append({"n": len(STATE["calls"]) + 1, "method": method, "path": path, "query": q, "body": body,
                                   "idempotency_key": self.headers.get("X-Idempotency-Key"),
                                   "authorization": self.headers.get("Authorization")})
            c = STATE["config"]
            if method == "POST" and path == "/v1/payments":
                key = self.headers.get("X-Idempotency-Key")
                if key and key in STATE["keys"]:
                    return self.reply(201, payment_json(STATE["payments"][STATE["keys"][key]]))
                b = body if isinstance(body, dict) else {}
                token = str(b.get("token") or "")
                mode = next((m for m in ("reject", "pending", "authorized", "error") if token.startswith(m)), c["mode"])
                if mode == "error":
                    return self.reply(500, {"message": "mpd forced error"})
                mode = {"reject": "rejected"}.get(mode, mode)
                received = round(float(b.get("transaction_amount", 0)) * 100)
                p = {"id": STATE["next_id"], "status": STATUS.get(mode, "approved"),
                     "amount_cents": c["amount_cents"] if c["amount_cents"] is not None else received,
                     "currency": c["currency"], "collector_id": c["collector_id"], "payment_type_id": c["payment_type_id"],
                     "external_reference": c["external_reference"] or b.get("external_reference"),
                     "payment_method_id": b.get("payment_method_id") or "visa"}
                STATE["next_id"] += 1
                STATE["payments"][p["id"]] = p
                if key:
                    STATE["keys"][key] = p["id"]
                return self.reply(201, payment_json(p))
            if method == "GET" and path == "/v1/payments/search":
                ref = (q.get("external_reference") or [""])[0]
                return self.reply(200, {"results": [payment_json(p) for p in STATE["payments"].values()
                                                    if p["external_reference"] == ref]})
            if method == "GET" and path.startswith("/v1/payments/"):
                p = STATE["payments"].get(int(path.rsplit("/", 1)[1]) if path.rsplit("/", 1)[1].isdigit() else -1)
                return self.reply(200, payment_json(p)) if p else self.reply(404, {"message": "payment not found"})
            self.reply(404, {"message": "mpd: unknown route"})

    def control(self, method, path, body):
        if path == "/_mpd/calls":
            if method == "DELETE":
                STATE.update(config=dict(DEFAULTS), calls=[], payments={}, keys={}, next_id=9000001)
                return self.reply(200, {"reset": True})
            return self.reply(200, STATE["calls"])
        if path == "/_mpd/payments" and method == "GET":
            return self.reply(200, [payment_json(p) for p in STATE["payments"].values()])
        if path == "/_mpd/config":
            if method == "POST" and isinstance(body, dict):
                STATE["config"].update(body)
            return self.reply(200, STATE["config"])
        if path.startswith("/_mpd/payments/") and method == "POST" and isinstance(body, dict):
            p = STATE["payments"].get(int(path.rsplit("/", 1)[1]))
            if not p:
                return self.reply(404, {"message": "payment not found"})
            if "status" in body:
                p["status"] = body["status"]
            if "amount_cents" in body:
                p["amount_cents"] = body["amount_cents"]
            return self.reply(200, payment_json(p))
        self.reply(404, {"message": "mpd: unknown control route"})

    def do_GET(self): self.handle_any("GET")
    def do_POST(self): self.handle_any("POST")
    def do_DELETE(self): self.handle_any("DELETE")


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", 8099), H).serve_forever()
