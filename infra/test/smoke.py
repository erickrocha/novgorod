#!/usr/bin/env python3
"""Smoke test of the HTTP kit: up -> seed -> Customer login -> public endpoint -> quote -> purchase -> payment (MPD) -> webhook
-> concurrent replay -> time shift -> reset -> down -v. Always tears the stack down (use --keep to leave it up)."""
import json, subprocess, sys, kit

def sh(*a):
    return subprocess.run(["docker", "compose", *a], cwd=kit.HERE).returncode

def step(title, res, want):
    code, body, ms = res
    print(f"[{'ok' if code in want else 'FAIL'}] {title}: HTTP {code} ({ms} ms) {json.dumps(body, ensure_ascii=False)[:230]}")
    if code not in want:
        raise SystemExit(f"smoke failed at: {title}")
    return body

def main():
    sh("down", "-v")
    try:
        if sh("up", "-d", "--build", "--wait", "kremlin", "mpd"):
            raise SystemExit("stack did not become healthy (docker compose logs kremlin)")
        print("== health:", kit.http("GET", "/")[:2])
        kit.seed(None)
        f = kit.fixtures()
        print("== login as C1:", kit.token("c1")[:24] + "...")
        step("public GET /api/public/products", kit.api("GET", "/api/public/products"), (200,))
        q = step("POST /checkout/quotes C1: A1 x1 + B1 x2, AD1", kit.api("POST", "/checkout/quotes", "c1", {"items": [{"skuId": f["skus"]["A1"], "quantity": 1}, {"skuId": f["skus"]["B1"], "quantity": 2}], "addressId": f["addresses"]["AD1"], "coupons": []}), (201,))
        print("   totals:", {k: q[k] for k in ("subtotalCents", "discountCents", "shippingCents", "totalCents")})
        body = {"quoteId": q["id"], "email": "c1@test.local", "phone": "11988887777"}
        p = step("POST /purchases", kit.api("POST", "/purchases", "c1", body, {"Idempotency-Key": "smoke-1"}), (201,))
        res = kit.burst([{"as": "c1", "path": "/purchases", "json": body, "headers": {"Idempotency-Key": "smoke-1"}}] * 5)
        print("   5 simultaneous replays of the same key:", sorted(r[0] for r in res), "-> purchases in DB:", kit.psql("SELECT count(*) FROM purchase", True, True).strip())
        step("POST payments/submit (MPD approved)", kit.api("POST", f"/purchases/{p['id']}/payments/submit", "c1", {"token": "tok-smoke", "paymentMethodId": "visa", "installments": 1}), (200, 503))
        print("   MPD calls:", [(c["method"], c["path"]) for c in kit.http("GET", "/_mpd/calls", base=kit.MPD)[1]])
        pay = kit.http("GET", "/_mpd/payments", base=kit.MPD)[1]
        if pay:
            step("webhook valid signature", kit.http("POST", f"/webhooks/mercado-pago?data.id={pay[0]['id']}", {"data": {"id": str(pay[0]["id"])}}, {"x-request-id": "r1", "x-signature": kit.sign(pay[0]["id"], "r1")}), (200, 503))
            step("webhook bad signature", kit.http("POST", f"/webhooks/mercado-pago?data.id={pay[0]['id']}", {"data": {"id": str(pay[0]["id"])}}, {"x-request-id": "r1", "x-signature": kit.sign(pay[0]["id"], "r1", "nope")}), (401,))
        print(kit.psql("SELECT p.id, p.status, pay.status AS payment_status, pay.amount_cents, p.total_cents FROM purchase p JOIN payment pay ON pay.purchase_id=p.id", True))
        kit.shift(type("A", (), {"what": "quote", "id": q["id"], "minutes": "20"}))
        step("expired quote -> purchase refused", kit.api("POST", "/purchases", "c1", {**body, "quoteId": q["id"]}, {"Idempotency-Key": "smoke-2"}), (409,))
        kit.reset(None)
        print("after reset: purchases =", kit.psql("SELECT count(*) FROM purchase", True, True).strip(), "| stock reserved sum =", kit.psql("SELECT sum(reserved) FROM sku_stock", True, True).strip())
        print("SMOKE OK")
    finally:
        if "--keep" not in sys.argv:
            sh("down", "-v")

main()
