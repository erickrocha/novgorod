#!/usr/bin/env python3
"""PagSeguro/PagBank test double (HTTP kit). Python stdlib only; state is in memory.

kremlin reaches it through PAGSEGURO_API_BASE_URL (honored only with PAYMENT_MOCK_ALLOWED=true). Endpoints the app uses
(application/src/infrastructure/pagseguro.rs):
  POST /charges                         charge  -> charge JSON (201)
  GET  /charges/{id}                    status
  GET  /charges?reference_id=...        search  -> {"charges": [...]}
  POST /charges/{id}/cancel             cancel  -> charge JSON (status CANCELED)
  GET  /charges/fees/calculate          installment plans: interest-free up to max_installments_no_interest, then 0.35% per extra installment
Control endpoints (not part of PagSeguro):
  GET    /_psd/calls                    every call received, oldest first (method, path, query, body, idempotency key)
  DELETE /_psd/calls                    reset calls, charges, config (charge ids keep counting)
  GET    /_psd/payments                 charges created so far
  GET    /_psd/config   POST /_psd/config
         {"mode": "approved|rejected|pending|authorized|error",   what the next charges answer (default approved)
          "amount_cents": null|int,       answer this amount instead of the one received (inconsistent-amount case)
          "currency": "BRL", "payment_type_id": "CREDIT_CARD", "external_reference": null|str}
  POST   /_psd/payments/{id}  {"status": "approved", "amount_cents": ...}   change a stored charge (status/search/webhook then see it)
A card token that starts with "reject", "pending", "authorized" or "error" overrides the mode for that charge.
A repeated x-idempotency-key returns the stored charge instead of creating another.
"""
import json, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

LOCK = threading.Lock()
DEFAULTS = {"mode": "approved", "amount_cents": None, "currency": "BRL", "payment_type_id": "CREDIT_CARD", "external_reference": None}
STATE = {"config": dict(DEFAULTS), "calls": [], "payments": {}, "keys": {}, "next_id": 1}
STATUS = {"approved": "PAID", "rejected": "DECLINED", "pending": "WAITING", "authorized": "AUTHORIZED", "canceled": "CANCELED"}


def status_of(value):
    return STATUS.get(str(value).lower(), str(value).upper())


def charge_json(p):
    return {"id": p["id"], "reference_id": p["reference_id"], "status": p["status"],
            "amount": {"value": p["amount_cents"], "currency": p["currency"]},
            "payment_method": {"type": p["payment_type_id"],
                               "card": {"brand": "visa", "last_digits": "4242", "exp_month": 12, "exp_year": 2030,
                                        "holder": {"name": "Test Holder"}}}}


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
            if path.startswith("/_psd/"):
                return self.control(method, path, body)
            STATE["calls"].append({"n": len(STATE["calls"]) + 1, "method": method, "path": path, "query": q, "body": body,
                                   "idempotency_key": self.headers.get("x-idempotency-key"),
                                   "authorization": self.headers.get("Authorization")})
            c = STATE["config"]
            if method == "POST" and path == "/charges":
                key = self.headers.get("x-idempotency-key")
                if key and key in STATE["keys"]:
                    return self.reply(201, charge_json(STATE["payments"][STATE["keys"][key]]))
                b = body if isinstance(body, dict) else {}
                card = (b.get("payment_method") or {}).get("card") or {}
                token = str(card.get("token") or card.get("encrypted") or "")
                mode = next((m for m in ("reject", "pending", "authorized", "error") if token.startswith(m)), c["mode"])
                if mode == "error":
                    return self.reply(500, {"error_messages": [{"code": "40002", "description": "psd forced error"}]})
                mode = {"reject": "rejected"}.get(mode, mode)
                received = int((b.get("amount") or {}).get("value", 0))
                p = {"id": "CHAR_%08d" % STATE["next_id"], "status": status_of(mode),
                     "amount_cents": c["amount_cents"] if c["amount_cents"] is not None else received,
                     "currency": c["currency"], "payment_type_id": c["payment_type_id"],
                     "reference_id": c["external_reference"] or b.get("reference_id")}
                STATE["next_id"] += 1
                STATE["payments"][p["id"]] = p
                if key:
                    STATE["keys"][key] = p["id"]
                return self.reply(201, charge_json(p))
            if method == "GET" and path == "/charges/fees/calculate":
                value = int((q.get("value") or ["0"])[0])
                free = int((q.get("max_installments_no_interest") or ["0"])[0])
                top = int((q.get("max_installments") or ["12"])[0])
                rows = []
                for n in range(1, top + 1):
                    total = value if n == 1 or n <= free else value + value * (n - free) * 35 // 10000
                    rows.append({"installments": n, "installment_value": total // n, "interest_free": n == 1 or n <= free,
                                 "amount": {"value": total, "currency": "BRL"}})
                brand = "elo" if q.get("credit_card_bin") else "visa"
                return self.reply(200, {"payment_methods": {"credit_card": {brand: {"installment_plans": rows}}}})
            if method == "GET" and path == "/charges":
                ref = (q.get("reference_id") or [""])[0]
                return self.reply(200, {"charges": [charge_json(p) for p in STATE["payments"].values() if p["reference_id"] == ref]})
            if method == "POST" and path.startswith("/charges/") and path.endswith("/cancel"):
                p = STATE["payments"].get(path.split("/")[2])
                if not p:
                    return self.reply(404, {"error_messages": [{"code": "40004"}]})
                p["status"] = "CANCELED"
                return self.reply(200, charge_json(p))
            if method == "GET" and path.startswith("/charges/"):
                p = STATE["payments"].get(path.rsplit("/", 1)[1])
                return self.reply(200, charge_json(p)) if p else self.reply(404, {"error_messages": [{"code": "40004"}]})
            self.reply(404, {"message": "psd: unknown route"})

    def control(self, method, path, body):
        if path == "/_psd/calls":
            if method == "DELETE":
                # ids keep counting: a real provider never repeats one, and kremlin stores them uniquely
                STATE.update(config=dict(DEFAULTS), calls=[], payments={}, keys={})
                return self.reply(200, {"reset": True})
            return self.reply(200, STATE["calls"])
        if path == "/_psd/payments" and method == "GET":
            return self.reply(200, [charge_json(p) for p in STATE["payments"].values()])
        if path == "/_psd/config":
            if method == "POST" and isinstance(body, dict):
                STATE["config"].update(body)
            return self.reply(200, STATE["config"])
        if path.startswith("/_psd/payments/") and method == "POST" and isinstance(body, dict):
            p = STATE["payments"].get(path.rsplit("/", 1)[1])
            if not p:
                return self.reply(404, {"message": "payment not found"})
            if "status" in body:
                p["status"] = status_of(body["status"])
            if "amount_cents" in body:
                p["amount_cents"] = body["amount_cents"]
            return self.reply(200, charge_json(p))
        self.reply(404, {"message": "psd: unknown control route"})

    def do_GET(self): self.handle_any("GET")
    def do_POST(self): self.handle_any("POST")
    def do_DELETE(self): self.handle_any("DELETE")


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", 8099), H).serve_forever()
