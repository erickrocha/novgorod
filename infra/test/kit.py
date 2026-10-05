#!/usr/bin/env python3
"""HTTP test kit for NOV-3 (T-CHK-01..03). Python 3 stdlib only; run from the host while the stack is up.

  ./kit.py seed                      fixtures E1 through the HTTP API (SQL only where the API cannot: see SQL_FALLBACK)
  ./kit.py reset                     back to the seeded state between cases (rows, stock, coupons, MPD)
  ./kit.py ids                       fixtures: ids of tenants, users, SKUs, addresses, coupons
  ./kit.py call METHOD PATH [--as WHO] [--json BODY] [-H 'K: V']   one request, prints status + body
  ./kit.py burst --repeat N METHOD PATH [...]  |  burst --spec '[{...},{...}]'   simultaneous requests
  ./kit.py coupon CODE TYPE VALUE [--tenant T1] [--field k=v ...]   POST /coupons as the tenant owner
  ./kit.py mpd approved|rejected|pending|authorized|error [--amount-cents N]   set the Mercado Pago double
  ./kit.py mpd-calls | mpd-payments | mpd-reset
  ./kit.py webhook PAYMENT_ID [--request-id X] [--bad]     signed POST /webhooks/mercado-pago
  ./kit.py evidence [purchase|orders|payment|allocation|coupon-reservation|redemption|stock|stock-reservation|quote|address|all] [PURCHASE_ID]
  ./kit.py db "SELECT ..."           read-only SQL (the session is read-only)
  ./kit.py shift quote|purchase|attempt|reservation ID MINUTES   age a row: its time column moves MINUTES into the past
  ./kit.py sql "UPDATE ..."          any SQL (writes allowed)

WHO: sysadmin, s1 (TenantOwner T1), s2 (TenantOwner T2), u1 (TenantUser T1), c1, c2, c3, anon.
Body, path and headers may use {{NAME}} for any fixture (A1, AD1, T1, C1, CP10 ...; see `ids`).
Values are throwaway and match docker-compose.yml; override with the TEST_* environment variables it reads.
"""
import argparse, concurrent.futures, hashlib, hmac, json, os, re, subprocess, sys, threading, time, urllib.error, urllib.request
from datetime import datetime, timedelta, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
BASE = os.environ.get("KIT_BASE_URL", f"http://localhost:{os.environ.get('TEST_KREMLIN_PORT', '8080')}")
MPD = os.environ.get("KIT_MPD_URL", f"http://localhost:{os.environ.get('TEST_MPD_PORT', '8099')}")
WEBHOOK_SECRET = os.environ.get("TEST_MP_WEBHOOK_SECRET", "test-mp-webhook-3a9c5e7b1d2f4860")
DB = (os.environ.get("TEST_DATABASE_USER", "novgorod_test"), os.environ.get("TEST_DATABASE_NAME", "novgorod_test"))
FIX = os.path.join(HERE, ".kit-fixtures.json")  # git-ignored

# who -> (email, password). Throwaway; the sysadmin pair is the one set in docker-compose.yml.
USERS = {"sysadmin": ("sysadmin@test.local", os.environ.get("TEST_SYSADMIN_PASSWORD", "SysAdmin-test-1")),
         "s1": ("s1@test.local", "Owner-test-1"), "s2": ("s2@test.local", "Owner-test-2"), "u1": ("u1@test.local", "Owner-test-1"),
         "c1": ("c1@test.local", "Cust-test-1"), "c2": ("c2@test.local", "Cust-test-2"), "c3": ("c3@test.local", "Cust-test-3")}
CPF = {"c1": "52998224725", "c2": "11144477735", "c3": "39053344705"}
OWNER_OF = {"T1": "s1", "T2": "s2"}
# product name, sku code, tenant, price, sku active, product active; PRO-1 has one SKU (P1), PRO-2 two (V1, V2)
SKUS = [("A1", "T1", 5000, 1, 1), ("A2", "T1", 1005, 1, 1), ("A3", "T1", 999, 1, 1), ("Z1", "T1", 0, 1, 1), ("X1", "T1", 2000, 0, 1),
        ("W1", "T1", 2000, 1, 0), ("P1", "T1", 1500, 1, 1), ("V1", "T1", 1000, 1, 1), ("V2", "T1", 1100, 1, 1), ("B1", "T2", 3000, 1, 1)]
PRODUCT_OF = {"P1": "PRO-1", "V1": "PRO-2", "V2": "PRO-2"}  # every other SKU has its own product
RATES = [("T1", "SP", 1500), ("T1", "RJ", 0), ("T2", "SP", 1000)]
ADDRESSES = [("AD1", "c1", "SP", "01001000"), ("AD2", "c1", "SP", "01310100"), ("AD3", "c1", "RJ", "20040020"),
             ("AD9", "c2", "SP", "04538132"), ("AD10", "c3", "SP", "01001000")]
STOCK = 10
SQL_FALLBACK = "customer_address rows and the TenantUser U1 are inserted by SQL (see seed())"


def coupons():
    """Standard coupon set (names from test-cases.md), posted by seed and again by reset."""
    now = datetime.now(timezone.utc).replace(tzinfo=None)
    iso = lambda d: d.isoformat(timespec="seconds")
    c = lambda code, t="PERCENTAGE", v=10, tenant="T1", **kw: dict(code=code, couponType=t, value=v, _tenant=tenant, **kw)
    return [c("CP10"), c("CP10b"), c("CP33", v=33), c("CF5000", "FIXED", 5000), c("CP100", v=100),
            c("CP100b", v=100, maxUsesPerCustomer=1), c("CP1U", maxUses=1), c("CIN", active=False),
            c("CFUT", startsAt=iso(now + timedelta(days=1))), c("CEXP", expiresAt=iso(now - timedelta(days=1))),
            c("CMIN", minOrderCents=6000), c("CB", tenant="T2")]


# ---------------------------------------------------------------- HTTP
_tokens, _tlock = {}, threading.Lock()


def http(method, path, body=None, headers=None, base=None, form=None, timeout=60):
    data, h = None, dict(headers or {})
    if form is not None:
        data = "&".join(f"{k}={v}" for k, v in form.items()).encode()
        h["Content-Type"] = "application/x-www-form-urlencoded"
    elif body is not None:
        data = body if isinstance(body, bytes) else (body if isinstance(body, str) else json.dumps(body)).encode()
        h.setdefault("Content-Type", "application/json")
    req = urllib.request.Request((base or BASE) + path, data=data, headers=h, method=method)
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            code, raw = r.status, r.read()
    except urllib.error.HTTPError as e:
        code, raw = e.code, e.read()
    txt = raw.decode(errors="replace")
    try:
        parsed = json.loads(txt) if txt else None
    except ValueError:
        parsed = txt
    return code, parsed, round((time.time() - t0) * 1000)


def token(who):
    if who in (None, "anon"):
        return None
    with _tlock:
        if who not in _tokens:
            email, pw = USERS[who]
            code, body, _ = http("POST", "/login", form={"email": email, "password": pw})
            if code != 200:
                sys.exit(f"login as {who} failed: {code} {body}")
            _tokens[who] = body["accessToken"]
        return _tokens[who]


def fixtures():
    try:
        return json.load(open(FIX))
    except OSError:
        return {}


def flat(extra=None):
    f = fixtures()
    m = {}
    for part in ("tenants", "skus", "addresses", "coupons", "customers", "users"):
        m.update({k: str(v) for k, v in f.get(part, {}).items()})
    m.update({k: str(v) for k, v in (extra or {}).items()})
    return m


def sub(text, extra=None):
    m = flat(extra)
    return re.sub(r"\{\{(\w+)\}\}", lambda x: m.get(x.group(1), x.group(0)), text) if isinstance(text, str) else text


def api(method, path, who=None, body=None, headers=None, extra=None):
    h = {k: sub(v, extra) for k, v in (headers or {}).items()}
    t = token(who)
    if t:
        h["Authorization"] = f"Bearer {t}"
    b = body if isinstance(body, (dict, list)) or body is None else sub(body, extra)
    if isinstance(b, (dict, list)):
        b = sub(json.dumps(b), extra)
    return http(method, sub(path, extra), b, h)


def ok(step, res, want=(200, 201, 204)):
    code, body, _ = res
    if code not in want:
        sys.exit(f"seed step '{step}' failed: {code} {body}")
    return body


# ---------------------------------------------------------------- DB (through the postgres container)
def psql(sql, readonly=False, tuples=False):
    cmd = ["docker", "compose", "exec", "-T"] + (["-e", "PGOPTIONS=-c default_transaction_read_only=on"] if readonly else []) + \
          ["postgres", "psql", "-U", DB[0], "-d", DB[1], "-v", "ON_ERROR_STOP=1", "-P", "pager=off"] + (["-At"] if tuples else []) + ["-c", sql]
    r = subprocess.run(cmd, cwd=HERE, capture_output=True, text=True)
    if r.returncode:
        sys.exit(r.stderr.strip() or "psql failed")
    return r.stdout


# ---------------------------------------------------------------- seed / reset
def seed(args):
    if int(psql("SELECT count(*) FROM tenant", tuples=True) or 0):
        print("already seeded; use `reset` or `docker compose down -v`")
        return
    f = {"tenants": {}, "users": {}, "customers": {}, "skus": {}, "addresses": {}, "coupons": {}}
    for i, (t, owner) in enumerate(OWNER_OF.items(), 1):
        r = ok(f"tenant {t}", api("POST", "/tenant", "sysadmin", dict(businessName=f"Seller {t}", taxId=f"1234567800010{i}", email=f"{t.lower()}@test.local",
               phone=f"1199999000{i}", addressLine1="Rua 1", locality="Sao Paulo", administrativeArea="SP", postalCode="01001000", countryCode="BR")))
        f["tenants"][t] = r["id"]
        email, pw = USERS[owner]
        f["users"][owner.upper()] = ok(f"owner {owner}", api("POST", "/user", "sysadmin", dict(name=f"Owner {t}", email=email, password=pw, enabled=True,
                                       role="TenantOwner", tenantId=r["id"])))["id"]
    ok("list T1", api("PUT", f"/tenant/{f['tenants']['T1']}/listing", "sysadmin", {"listed": True}))
    for who in ("c1", "c2", "c3"):
        email, pw = USERS[who]
        ok(f"signup {who}", api("POST", "/signup", None, dict(name=f"Customer {who.upper()}", email=email, password=pw, cpf=CPF[who], phone="11988887777")))
        f["customers"][who.upper()] = int(psql(f"SELECT id FROM customer WHERE email='{email}'", tuples=True))
    # SQL_FALLBACK 1: POST /customer-addresses as a Customer fails (the tenant hook writes tenant_id NULL, column is NOT NULL); signup with an address fails the same way.
    t1 = f["tenants"]["T1"]
    for name, who, uf, cep in ADDRESSES:
        f["addresses"][name] = int(psql(f"INSERT INTO customer_address (uuid,tenant_id,customer_id,label,recipient,address_line1,locality,administrative_area,postal_code,country_code,"
                                        f"is_default,created_at,updated_at) VALUES (gen_random_uuid(),{t1},{f['customers'][who.upper()]},'{name}','Customer {who.upper()}','Rua {name} 100','Cidade {uf}','{uf}','{cep}','BR',"
                                        f"{'true' if name in ('AD1', 'AD9', 'AD10') else 'false'},now(),now()) RETURNING id", tuples=True).split()[0])
    # SQL_FALLBACK 2: no API creates a TenantUser (POST /user as owner forces TenantOwner); reuse S1's password hash.
    f["users"]["U1"] = int(psql(f"INSERT INTO \"user\" (uuid,tenant_id,name,email,password,first_login,enabled,role,created_at,updated_at) "
                                f"SELECT gen_random_uuid(),tenant_id,'User T1','{USERS['u1'][0]}',password,false,true,'TenantUser',now(),now() FROM \"user\" WHERE email='{USERS['s1'][0]}' RETURNING id",
                                tuples=True).split()[0])
    for t, uf, price in RATES:
        ok(f"rate {t} {uf}", api("POST", "/shipping-rates", OWNER_OF[t], dict(uf=uf, priceCents=price)))
    prods = {}
    for code, t, price, sku_active, prod_active in SKUS:
        pname = PRODUCT_OF.get(code, code)
        if (pname, t) not in prods:
            prods[(pname, t)] = ok(f"product {pname}", api("POST", "/products", OWNER_OF[t], dict(name=f"Product {pname}", slug=f"product-{pname.lower()}", active=bool(prod_active),
                                   ncm="22042100", origemMercadoria=0)))["id"]
        sku = ok(f"sku {code}", api("POST", "/skus", OWNER_OF[t], dict(productId=prods[(pname, t)], code=code, variantKey=code.lower(), priceCents=price, active=bool(sku_active))))["id"]
        f["skus"][code] = sku
        ok(f"stock {code}", api("POST", "/sku-stocks", OWNER_OF[t], dict(skuId=sku, quantity=STOCK)))
    json.dump(f, open(FIX, "w"), indent=1)
    make_coupons(f)
    print(f"seeded: {json.dumps(f)}\nsql fallbacks: {SQL_FALLBACK}")


def make_coupons(f):
    for c in coupons():
        tenant = c.pop("_tenant")
        r = ok(f"coupon {c['code']}", api("POST", "/coupons", OWNER_OF[tenant], c))
        f["coupons"][c["code"]] = r["id"]
        f.setdefault("coupon_tenants", {})[c["code"]] = tenant
    json.dump(f, open(FIX, "w"), indent=1)


def reset(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    tables = ["checkout_quote", "coupon_redemption", "checkout_coupon_reservation", "credit_card_details", "payment_transaction", "payment_allocation", "payment",
              "order_status_history", "order_address", "order_item", "orders", "purchase", "cart_item", "cart", "coupon"]
    if psql("SELECT to_regclass('checkout_stock_reservation') IS NOT NULL", tuples=True).strip() == "t":
        tables.insert(0, "checkout_stock_reservation")
    psql(f"TRUNCATE {', '.join(tables)} RESTART IDENTITY CASCADE; UPDATE sku_stock SET quantity={STOCK}, reserved=0;"
)
    f["coupons"] = {}
    _tokens.clear()
    make_coupons(f)
    http("DELETE", "/_mpd/calls", base=MPD)
    print("reset: purchases, quotes, carts, coupons, reservations cleared; stock restored; coupons recreated; MPD reset")


def tchk08(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    reset(args)
    c1, t1, a1 = f["customers"]["C1"], f["tenants"]["T1"], f["skus"]["A1"]

    def quote(code, address, headers=None):
        return api("POST", "/checkout/quotes", "c1", dict(
            items=[dict(skuId=a1, quantity=1)], addressId=f["addresses"][address],
            coupons=[dict(tenantId=t1, code=code)],
        ), headers)

    def purchase(quote_id, key, headers=None):
        return api("POST", "/purchases", "c1", dict(
            quoteId=quote_id, email=USERS["c1"][0], phone="11988887777",
        ), {"Idempotency-Key": key, **(headers or {})})

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-08 failed: {message}")

    code, body, _ = quote("CP100", "AD3")
    require(code == 201 and body["totalCents"] == 0 and body["discountCents"] == 5000,
            f"marketplace quote: HTTP {code} {body}")
    code, body, _ = purchase(body["id"], "tchk08-marketplace")
    require(code == 201 and body["status"] == "paid", f"marketplace purchase: HTTP {code} {body}")
    counts = psql(
        "SELECT (SELECT count(*) FROM coupon_redemption WHERE order_id IN "
        f"(SELECT id FROM orders WHERE purchase_id={body['id']}) AND tenant_id={t1} "
        f"AND coupon_id={f['coupons']['CP100']} AND customer_id={c1}), "
        f"(SELECT count(*) FROM checkout_coupon_reservation WHERE purchase_id={body['id']} "
        f"AND coupon_id={f['coupons']['CP100']} AND status='redeemed')",
        tuples=True,
    ).strip()
    require(counts == "1|1", f"marketplace redemption/reservation: {counts}")

    store_headers = {"x-tenant-id": str(t1)}
    code, body, _ = quote("CP100b", "AD3", store_headers)
    require(code == 201 and body["totalCents"] == 0, f"store-mode quote: HTTP {code} {body}")
    code, body, _ = purchase(body["id"], "tchk08-store", store_headers)
    require(code == 201 and body["status"] == "paid", f"store-mode purchase: HTTP {code} {body}")
    counts = psql(
        "SELECT (SELECT count(*) FROM coupon_redemption WHERE order_id IN "
        f"(SELECT id FROM orders WHERE purchase_id={body['id']}) AND tenant_id={t1} "
        f"AND coupon_id={f['coupons']['CP100b']} AND customer_id={c1}), "
        f"(SELECT count(*) FROM checkout_coupon_reservation WHERE purchase_id={body['id']} "
        f"AND coupon_id={f['coupons']['CP100b']} AND status='redeemed')",
        tuples=True,
    ).strip()
    require(counts == "1|1", f"store-mode redemption/reservation: {counts}")

    code, body, _ = quote("CP10", "AD1")
    require(code == 201 and body["totalCents"] > 0, f"payment quote: HTTP {code} {body}")
    code, body, _ = purchase(body["id"], "tchk08-payment")
    require(code == 201 and body["status"] == "pending_payment", f"pending purchase: HTTP {code} {body}")
    http("POST", "/_mpd/config", {"mode": "approved"}, base=MPD)
    code, body, _ = api("POST", f"/purchases/{body['id']}/payments/submit", "c1", dict(
        token="tok-tchk08-approved", paymentMethodId="visa", installments=1,
    ))
    require(code == 200 and body["status"] == "captured", f"Customer payment confirmation: HTTP {code} {body}")
    counts = psql(
        "SELECT (SELECT count(*) FROM coupon_redemption WHERE order_id IN "
        f"(SELECT id FROM orders WHERE purchase_id={body['purchaseId']}) AND tenant_id={t1} "
        f"AND coupon_id={f['coupons']['CP10']} AND customer_id={c1}), "
        f"(SELECT count(*) FROM checkout_coupon_reservation WHERE purchase_id={body['purchaseId']} "
        f"AND coupon_id={f['coupons']['CP10']} AND status='redeemed'), "
        f"(SELECT count(*) FROM payment WHERE purchase_id={body['purchaseId']} AND status='captured')",
        tuples=True,
    ).strip()
    require(counts == "1|1|1", f"payment redemption/reservation/capture: {counts}")
    print("T-CHK-08 PASS: Customer marketplace and store-mode zero-total purchases, and approved coupon payment confirmed through HTTP auth")


def tchk11(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    a1 = f["skus"]["A1"]

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-11 failed: {message}")

    def create_purchase(key):
        code, quote_body, _ = api("POST", "/checkout/quotes", "c1", dict(
            items=[dict(skuId=a1, quantity=3)],
            addressId=f["addresses"]["AD1"], coupons=[],
        ))
        require(code == 201, f"quote: HTTP {code} {quote_body}")
        code, body, _ = api("POST", "/purchases", "c1", dict(
            quoteId=quote_body["id"], email=USERS["c1"][0], phone="11988887777",
        ), {"Idempotency-Key": key})
        require(code == 201 and body["status"] == "pending_payment",
                f"purchase: HTTP {code} {body}")
        return body["id"]

    def inventory(purchase_id):
        return psql(
            f"SELECT (SELECT status FROM purchase WHERE id={purchase_id}) || '|' || "
            f"(SELECT status FROM payment WHERE purchase_id={purchase_id}) || '|' || "
            f"(SELECT quantity FROM sku_stock WHERE sku_id={a1}) || '|' || "
            f"(SELECT reserved FROM sku_stock WHERE sku_id={a1}) || '|' || "
            f"(SELECT status FROM checkout_stock_reservation WHERE purchase_id={purchase_id})",
            tuples=True,
        ).strip()

    reset(args)
    purchase_id = create_purchase("tchk11-captured")
    http("POST", "/_mpd/config", {"mode": "approved"}, base=MPD)
    code, body, _ = api("POST", f"/purchases/{purchase_id}/payments/submit", "c1", dict(
        token="tok-tchk11-approved", paymentMethodId="visa", installments=1,
    ))
    require(code == 200 and body["status"] == "captured",
            f"approved submit: HTTP {code} {body}")
    expected = "paid|captured|7|0|consumed"
    require(inventory(purchase_id) == expected, f"captured inventory: {inventory(purchase_id)}")
    code, body, _ = api("GET", f"/purchases/{purchase_id}/payments/status", "c1")
    require(code == 200 and body["status"] == "captured", f"captured replay: HTTP {code} {body}")
    require(inventory(purchase_id) == expected, "captured replay changed stock")

    reset(args)
    purchase_id = create_purchase("tchk11-failed")
    http("POST", "/_mpd/config", {"mode": "rejected"}, base=MPD)
    code, body, _ = api("POST", f"/purchases/{purchase_id}/payments/submit", "c1", dict(
        token="tok-tchk11-rejected", paymentMethodId="visa", installments=1,
    ))
    require(code == 200 and body["status"] == "failed", f"rejected submit: HTTP {code} {body}")
    expected = "payment_failed|failed|10|0|released"
    require(inventory(purchase_id) == expected, f"failed inventory: {inventory(purchase_id)}")
    code, body, _ = api("GET", f"/purchases/{purchase_id}/payments/status", "c1")
    require(code == 200 and body["status"] == "failed", f"failed status replay: HTTP {code} {body}")
    provider_payments = http("GET", "/_mpd/payments", base=MPD)[1]
    payment_id = provider_payments[-1]["id"]
    request_id = f"tchk11-{payment_id}"
    signature = sign(payment_id, request_id)
    code, body, _ = http(
        "POST", f"/webhooks/mercado-pago?data.id={payment_id}",
        {"action": "payment.updated", "type": "payment", "data": {"id": str(payment_id)}},
        {"x-signature": signature, "x-request-id": request_id},
    )
    require(code == 200, f"failed webhook replay: HTTP {code} {body}")
    require(inventory(purchase_id) == expected, "failed replay changed stock")
    print("T-CHK-11 PASS: captured stock consumed once; failed stock released once across status and webhook replays")


def tchk12(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    c1, t1, a1 = f["customers"]["C1"], f["tenants"]["T1"], f["skus"]["A1"]

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-12 failed: {message}")

    def create_purchase(key, quantity):
        code, quote_body, _ = api("POST", "/checkout/quotes", "c1", dict(
            items=[dict(skuId=a1, quantity=quantity)],
            addressId=f["addresses"]["AD1"],
            coupons=[dict(tenantId=t1, code="CP10")],
        ))
        require(code == 201, f"quote: HTTP {code} {quote_body}")
        code, body, _ = api("POST", "/purchases", "c1", dict(
            quoteId=quote_body["id"], email=USERS["c1"][0], phone="11988887777",
        ), {"Idempotency-Key": key})
        require(code == 201, f"purchase: HTTP {code} {body}")
        return body["id"]

    def payment_state(purchase_id):
        return psql(
            f"SELECT (SELECT status FROM payment WHERE purchase_id={purchase_id}) || '|' || "
            f"(SELECT status FROM checkout_stock_reservation WHERE purchase_id={purchase_id}) || '|' || "
            f"(SELECT status FROM checkout_coupon_reservation WHERE purchase_id={purchase_id})",
            tuples=True,
        ).strip()

    reset(args)
    never_submitted = create_purchase("tchk12-unsubmitted", 2)
    in_flight = create_purchase("tchk12-in-flight", 1)
    code, body, _ = api("POST", f"/purchases/{in_flight}/payments/submit", "c1", dict(
        token="pending-tchk12", paymentMethodId="visa", installments=1,
    ))
    require(code == 200 and body["status"] == "pending", f"pending submit: HTTP {code} {body}")
    psql(
        "UPDATE purchase SET created_at=CURRENT_TIMESTAMP-interval '20 minutes' "
        f"WHERE id IN ({never_submitted},{in_flight}); "
        "UPDATE checkout_stock_reservation SET expires_at=CURRENT_TIMESTAMP-interval '20 minutes' "
        f"WHERE purchase_id IN ({never_submitted},{in_flight}); "
        "UPDATE checkout_coupon_reservation SET expires_at=CURRENT_TIMESTAMP-interval '20 minutes' "
        f"WHERE purchase_id IN ({never_submitted},{in_flight});"
    )

    expected_unsubmitted = "pending_provider|reserved|reserved"
    expected_in_flight = "pending|reserved|reserved"
    if args.expect_disabled:
        deadline = time.monotonic() + 32
        while time.monotonic() < deadline:
            time.sleep(0.5)
        require(payment_state(never_submitted) == expected_unsubmitted,
                f"disabled job changed unsubmitted payment: {payment_state(never_submitted)}")
        require(payment_state(in_flight) == expected_in_flight,
                f"disabled job changed in-flight payment: {payment_state(in_flight)}")
        require(psql("SELECT reserved FROM sku_stock WHERE sku_id=1", tuples=True).strip() == "3",
                "disabled job changed reserved stock")
        print("T-CHK-12 PASS: disabled switch preserved expired stock and coupon reservations")
        return

    deadline = time.monotonic() + 35
    while time.monotonic() < deadline:
        if payment_state(never_submitted) == "pending_provider|released|released":
            break
        time.sleep(0.5)
    require(payment_state(never_submitted) == "pending_provider|released|released",
            f"expired unsubmitted reservation not released: {payment_state(never_submitted)}")
    require(payment_state(in_flight) == expected_in_flight,
            f"in-flight payment reservation changed: {payment_state(in_flight)}")
    require(psql("SELECT quantity||'|'||reserved FROM sku_stock WHERE sku_id=1", tuples=True).strip() == "10|1",
            "expiry job did not release only the unsubmitted stock")
    print("T-CHK-12 PASS: expired unsubmitted reservation and coupon released; in-flight payment remained reserved")


def tchk13(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    a1 = f["skus"]["A1"]

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-13 failed: {message}")

    def create_purchase(key, quantity):
        code, quote_body, _ = api("POST", "/checkout/quotes", "c1", dict(
            items=[dict(skuId=a1, quantity=quantity)],
            addressId=f["addresses"]["AD1"], coupons=[],
        ))
        require(code == 201, f"quote: HTTP {code} {quote_body}")
        code, body, _ = api("POST", "/purchases", "c1", dict(
            quoteId=quote_body["id"], email=USERS["c1"][0], phone="11988887777",
        ), {"Idempotency-Key": key})
        require(code == 201, f"purchase: HTTP {code} {body}")
        return body["id"]

    def submit(purchase_id, token_value):
        return api("POST", f"/purchases/{purchase_id}/payments/submit", "c1", dict(
            token=token_value, paymentMethodId="visa", installments=1,
        ))

    def calls():
        return len(http("GET", "/_mpd/calls", base=MPD)[1])

    def state(purchase_id):
        return psql(
            f"SELECT (SELECT status FROM payment WHERE purchase_id={purchase_id}) || '|' || "
            f"(SELECT quantity FROM sku_stock WHERE sku_id={a1}) || '|' || "
            f"(SELECT reserved FROM sku_stock WHERE sku_id={a1}) || '|' || "
            f"(SELECT status FROM checkout_stock_reservation WHERE purchase_id={purchase_id})",
            tuples=True,
        ).strip()

    reset(args)
    purchase_id = create_purchase("tchk13-expired", 3)
    http("POST", "/_mpd/config", {"mode": "rejected"}, base=MPD)
    code, body, _ = submit(purchase_id, "tok-tchk13-reject")
    require(code == 200 and body["status"] == "failed", f"initial failed attempt: HTTP {code} {body}")
    require(state(purchase_id) == "failed|10|0|released", f"initial failure state: {state(purchase_id)}")
    kit_sql = f"UPDATE purchase SET created_at=CURRENT_TIMESTAMP-interval '20 minutes' WHERE id={purchase_id}"
    psql(kit_sql)
    before_calls = calls()
    code, body, _ = submit(purchase_id, "tok-tchk13-expired")
    require(code == 409, f"expired retry: HTTP {code} {body}")
    require(calls() == before_calls, "expired retry reached MPD")
    require(state(purchase_id) == "failed|10|0|released", f"expired retry changed state: {state(purchase_id)}")

    psql(f"UPDATE purchase SET created_at=CURRENT_TIMESTAMP-interval '5 minutes' WHERE id={purchase_id}")
    http("POST", "/_mpd/config", {"mode": "approved"}, base=MPD)
    code, body, _ = submit(purchase_id, "tok-tchk13-retry-approved")
    require(code == 200 and body["status"] == "captured", f"retry capture: HTTP {code} {body}")
    require(state(purchase_id) == "captured|7|0|consumed", f"retry capture state: {state(purchase_id)}")

    insufficient = create_purchase("tchk13-insufficient", 3)
    http("POST", "/_mpd/config", {"mode": "rejected"}, base=MPD)
    code, body, _ = submit(insufficient, "tok-tchk13-reject-b")
    require(code == 200 and body["status"] == "failed", f"second initial failure: HTTP {code} {body}")
    psql(f"UPDATE sku_stock SET quantity=2 WHERE sku_id={a1}; UPDATE purchase SET created_at=CURRENT_TIMESTAMP-interval '5 minutes' WHERE id={insufficient}")
    http("POST", "/_mpd/config", {"mode": "approved"}, base=MPD)
    before_calls = calls()
    code, body, _ = submit(insufficient, "tok-tchk13-insufficient-retry")
    require(code == 409, f"insufficient retry: HTTP {code} {body}")
    require(calls() == before_calls, "insufficient retry reached MPD")
    require(state(insufficient) == "failed|2|0|released", f"insufficient retry changed state: {state(insufficient)}")

    in_flight = create_purchase("tchk13-in-flight", 1)
    code, body, _ = submit(in_flight, "pending-tchk13-flight")
    require(code == 200 and body["status"] == "pending", f"pending attempt: HTTP {code} {body}")
    psql(f"UPDATE purchase SET created_at=CURRENT_TIMESTAMP-interval '20 minutes' WHERE id={in_flight}")
    before_calls = calls()
    code, body, _ = submit(in_flight, "pending-tchk13-replay")
    require(code == 200 and body["status"] == "pending", f"in-flight replay: HTTP {code} {body}")
    require(calls() == before_calls, "in-flight replay reached MPD again")
    require(state(in_flight) == "pending|2|1|reserved", f"in-flight reservation changed: {state(in_flight)}")
    print("T-CHK-13 PASS: 15-minute window, released-stock retry, insufficient-stock refusal, and in-flight attempt")


def tchk14(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    a1, t1 = f["skus"]["A1"], f["tenants"]["T1"]

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-14 failed: {message}")

    def create_purchase(customer, address, key):
        code, quote_body, _ = api("POST", "/checkout/quotes", customer, dict(
            items=[dict(skuId=a1, quantity=3)],
            addressId=f["addresses"][address], coupons=[],
        ))
        require(code == 201, f"quote: HTTP {code} {quote_body}")
        code, body, _ = api("POST", "/purchases", customer, dict(
            quoteId=quote_body["id"], email=USERS[customer][0], phone="11988887777",
        ), {"Idempotency-Key": key})
        require(code == 201, f"purchase: HTTP {code} {body}")
        return body["id"]

    def fail_payment(purchase_id):
        http("POST", "/_mpd/config", {"mode": "rejected"}, base=MPD)
        code, body, _ = api("POST", f"/purchases/{purchase_id}/payments/submit", "c1", dict(
            token=f"tok-tchk14-reject-{purchase_id}", paymentMethodId="visa", installments=1,
        ))
        require(code == 200 and body["status"] == "failed", f"initial reject: HTTP {code} {body}")
        payment = http("GET", "/_mpd/payments", base=MPD)[1][-1]
        return payment["id"]

    def approve_late(payment_id, purchase_id):
        http("POST", f"/_mpd/payments/{payment_id}", {"status": "approved"}, base=MPD)
        request_id = f"tchk14-{payment_id}"
        code, body, _ = http(
            "POST", f"/webhooks/mercado-pago?data.id={payment_id}",
            {"action": "payment.updated", "type": "payment", "data": {"id": str(payment_id)}},
            {"x-signature": sign(payment_id, request_id), "x-request-id": request_id},
        )
        require(code == 200, f"late approved webhook: HTTP {code} {body}")
        state = psql(
            f"SELECT (SELECT status FROM purchase WHERE id={purchase_id}) || '|' || "
            f"(SELECT status FROM payment WHERE purchase_id={purchase_id}) || '|' || "
            f"(SELECT quantity FROM sku_stock WHERE sku_id={a1}) || '|' || "
            f"(SELECT reserved FROM sku_stock WHERE sku_id={a1}) || '|' || "
            f"(SELECT status FROM checkout_stock_reservation WHERE purchase_id={purchase_id})",
            tuples=True,
        ).strip()
        return state

    reset(args)
    purchase_id = create_purchase("c1", "AD1", "tchk14-late-available")
    payment_id = fail_payment(purchase_id)
    require(psql(f"SELECT status FROM checkout_stock_reservation WHERE purchase_id={purchase_id}", tuples=True).strip() == "released",
            "first purchase reservation was not released")
    available_state = approve_late(payment_id, purchase_id)
    require(available_state == "paid|captured|7|0|consumed", f"available late capture: {available_state}")

    reset(args)
    failed_purchase = create_purchase("c1", "AD1", "tchk14-late-oversold")
    payment_id = fail_payment(failed_purchase)
    psql(f"UPDATE sku_stock SET quantity=3,reserved=0 WHERE sku_id={a1}")
    other_purchase = create_purchase("c2", "AD9", "tchk14-other-buyer")
    require(psql(f"SELECT quantity||'|'||reserved FROM sku_stock WHERE sku_id={a1}", tuples=True).strip() == "3|3",
            "second customer did not reserve all available stock")
    marker = f"late payment capture oversold, purchase_id={failed_purchase}, sku_id={a1}, quantity=3, manual refund"
    logs_before = subprocess.run(
        ["docker", "compose", "logs", "--since=1m", "kremlin"], cwd=HERE,
        capture_output=True, text=True, check=False,
    ).stdout.count(marker)
    oversold_state = approve_late(payment_id, failed_purchase)
    require(oversold_state == "paid|captured|3|3|released", f"oversold late capture: {oversold_state}")
    logs_after = subprocess.run(
        ["docker", "compose", "logs", "--since=1m", "kremlin"], cwd=HERE,
        capture_output=True, text=True, check=False,
    ).stdout.count(marker)
    require(logs_after == logs_before + 1, f"expected one oversell error log; before={logs_before}, after={logs_after}")
    require(psql(f"SELECT status FROM purchase WHERE id={other_purchase}", tuples=True).strip() == "pending_payment",
            "other customer's purchase changed during late capture")
    print("T-CHK-14 PASS: late capture consumed available stock; oversold capture confirmed payment, preserved inventory, and logged once")


def tchk15(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    tenant_id = f["tenants"]["T1"]

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-15 failed: {message}")

    reset(args)
    valid = dict(code="lowercase-valid", couponType="percentage", value=10)
    code, body, _ = api("POST", "/coupons", "s1", {"tenantId": tenant_id, **valid})
    require(code == 201 and body["couponType"] == "PERCENTAGE",
            f"lowercase type normalization: HTTP {code} {body}")
    coupon_id = body["id"]

    invalid_cases = [
        ("bad-create-percentage-high", dict(couponType="PERCENTAGE", value=101)),
        ("bad-create-percentage-low", dict(couponType="PERCENTAGE", value=-1)),
        ("bad-create-fixed", dict(couponType="FIXED", value=-5)),
        ("bad-create-type", dict(couponType="BOGUS", value=10)),
        ("bad-create-minimum", dict(couponType="PERCENTAGE", value=10, minOrderCents=-1)),
        ("bad-create-max", dict(couponType="PERCENTAGE", value=10, maxUses=-1)),
        ("bad-create-customer-max", dict(couponType="PERCENTAGE", value=10, maxUsesPerCustomer=-1)),
    ]
    for code_value, values in invalid_cases:
        payload = {"tenantId": tenant_id, "code": code_value, "couponType": "PERCENTAGE", "value": 10, **values}
        status, response, _ = api("POST", "/coupons", "s1", payload)
        require(status == 400, f"invalid create {code_value}: HTTP {status} {response}")
        exists = psql(f"SELECT count(*) FROM coupon WHERE tenant_id={tenant_id} AND code='{code_value.upper()}'", tuples=True).strip()
        require(exists == "0", f"invalid create persisted {code_value}")

    for code_value, values in invalid_cases:
        payload = {"tenantId": tenant_id, "code": "changed", "couponType": "PERCENTAGE", "value": 10, **values}
        status, response, _ = api("PUT", f"/coupons/{coupon_id}", "s1", payload)
        require(status == 400, f"invalid update {code_value}: HTTP {status} {response}")

    stored = psql(
        f"SELECT code||'|'||coupon_type||'|'||value FROM coupon WHERE id={coupon_id}", tuples=True
    ).strip()
    require(stored == "LOWERCASE-VALID|PERCENTAGE|10", f"invalid update changed existing row: {stored}")
    print("T-CHK-15 PASS: normalized valid coupon; invalid type, value, and limits rejected on create/update")


def tchk16(args):
    f = fixtures()
    if not f:
        sys.exit("no fixtures; run `seed` first")
    t1, c1 = f["tenants"]["T1"], f["customers"]["C1"]

    def require(condition, message):
        if not condition:
            sys.exit(f"T-CHK-16 failed: {message}")

    reset(args)
    code, quote_body, _ = api("POST", "/checkout/quotes", "c1", dict(
        items=[dict(skuId=f["skus"]["A1"], quantity=1)],
        addressId=f["addresses"]["AD1"],
        coupons=[dict(tenantId=t1, code="CP10")],
    ))
    require(code == 201, f"quote: HTTP {code} {quote_body}")
    code, purchase_body, _ = api("POST", "/purchases", "c1", dict(
        quoteId=quote_body["id"], email=USERS["c1"][0], phone="11988887777",
    ), {"Idempotency-Key": "tchk16-order"})
    require(code == 201, f"purchase: HTTP {code} {purchase_body}")
    order_id = purchase_body["orders"][0]["id"]
    payload = dict(
        tenantId=t1,
        couponId=f["coupons"]["CP10"],
        orderId=order_id,
        customerId=c1,
    )

    for actor in ("s1", "u1", "c1"):
        code, body, _ = api("POST", "/coupon-redemptions", actor, payload)
        require(code == 403, f"{actor} should be forbidden: HTTP {code} {body}")
    require(psql("SELECT count(*) FROM coupon_redemption", tuples=True).strip() == "0",
            "unauthorized requests inserted coupon redemption")

    code, body, _ = api("POST", "/coupon-redemptions", "sysadmin", payload)
    require(code == 201 and body["tenantId"] == t1,
            f"SysAdmin write: HTTP {code} {body}")
    stored = psql(
        f"SELECT tenant_id||'|'||coupon_id||'|'||order_id||'|'||customer_id FROM coupon_redemption WHERE order_id={order_id}",
        tuples=True,
    ).strip()
    require(stored == f"{t1}|{f['coupons']['CP10']}|{order_id}|{c1}",
            f"SysAdmin row mismatch: {stored}")
    print("T-CHK-16 PASS: TenantOwner, TenantUser, and Customer denied; SysAdmin created the tenant-scoped redemption")


# ---------------------------------------------------------------- driver
def show(res):
    code, body, ms = res
    print(f"HTTP {code} ({ms} ms)")
    print(json.dumps(body, indent=1, ensure_ascii=False) if isinstance(body, (dict, list)) else body)


def hdrs(lst):
    return {k.strip(): v.strip() for k, v in (x.split(":", 1) for x in lst or [])}


def burst(requests):
    """requests: list of {as, method, path, headers, json}; every {{i}} is the request index. All fired at once."""
    gate, out = threading.Barrier(len(requests)), [None] * len(requests)

    def run(i, r):
        token(r.get("as"))  # log in before the gate so only the request itself is timed
        gate.wait()
        out[i] = api(r.get("method", "POST"), r["path"], r.get("as"), r.get("json"), r.get("headers"), {"i": i})

    with concurrent.futures.ThreadPoolExecutor(len(requests)) as ex:
        list(ex.map(lambda p: run(*p), enumerate(requests)))
    return out


# ---------------------------------------------------------------- Mercado Pago: webhook signing
def sign(data_id, request_id, secret=WEBHOOK_SECRET, ts=None):
    """x-signature as checkout_payment_endpoint::verify_signature expects: HMAC-SHA256 over id:{data_id};request-id:{request_id};ts:{ts};"""
    ts = str(ts or int(time.time() * 1000))
    mac = hmac.new(secret.encode(), f"id:{data_id};request-id:{request_id};ts:{ts};".encode(), hashlib.sha256).hexdigest()
    return f"ts={ts},v1={mac}"


def webhook(args):
    rid = args.request_id or f"kit-{int(time.time() * 1000)}"
    sig = sign(args.payment_id, rid, "wrong-secret" if args.bad else WEBHOOK_SECRET)
    body = {"action": "payment.updated", "type": "payment", "data": {"id": str(args.payment_id)}}
    show(http("POST", f"/webhooks/mercado-pago?data.id={args.payment_id}", body, {"x-signature": sig, "x-request-id": rid}))


# ---------------------------------------------------------------- evidence (read-only)
EVIDENCE = {
    "purchase": "SELECT id,customer_id,status,subtotal_cents,discount_cents,shipping_cents,total_cents,idempotency_key,created_at FROM purchase {p:id} ORDER BY id",
    "orders": "SELECT id,purchase_id,tenant_id,status,payment_status,subtotal_cents,discount_cents,shipping_cents,total_cents,coupon_id,coupon_code FROM orders {p:purchase_id} ORDER BY id",
    "payment": "SELECT id,purchase_id,status,method,amount_cents,currency,gateway_provider,gateway_reference,attempt_key,attempt_started_at,attempt_error FROM payment {p:purchase_id} ORDER BY id",
    "allocation": "SELECT * FROM payment_allocation {p:purchase_id} ORDER BY 1",
    "coupon-reservation": "SELECT * FROM checkout_coupon_reservation {p:purchase_id} ORDER BY id",
    "redemption": "SELECT id,tenant_id,coupon_id,order_id,customer_id,created_at,created_by FROM coupon_redemption ORDER BY id",
    "stock": "SELECT s.id,s.tenant_id,k.code,s.quantity,s.reserved FROM sku_stock s JOIN sku k ON k.id=s.sku_id ORDER BY s.id",
    "quote": "SELECT id,customer_id,purchase_id,expires_at,created_at FROM checkout_quote ORDER BY id",
    "address": "SELECT customer_id,md5(string_agg(c::text,'|' ORDER BY id)) AS checksum, count(*) FROM customer_address c GROUP BY customer_id ORDER BY customer_id",
}


def evidence(args):
    names = list(EVIDENCE) + ["stock-reservation"] if args.what == "all" else [args.what]
    for n in names:
        print(f"-- {n}")
        if n == "stock-reservation":  # table arrives with SR-CHK-014 (T-CHK-09); tolerate its absence
            if psql("SELECT to_regclass('checkout_stock_reservation') IS NOT NULL", tuples=True).strip() != "t":
                print("(table checkout_stock_reservation does not exist yet)")
                continue
            sql = "SELECT * FROM checkout_stock_reservation {p:purchase_id} ORDER BY 1"
        else:
            sql = EVIDENCE[n]
        flt = re.search(r"\{p:(\w+)\}", sql)
        where = f"WHERE {flt.group(1)}={int(args.id)}" if flt and args.id else ""
        print(psql(re.sub(r"\{p:\w+\}", where, sql), readonly=True))


def shift(args):
    m = int(args.minutes)
    iv = f"interval '{m} minutes'"
    if args.what == "reservation":  # stock table is optional (does not exist yet); the coupon reservation table always does
        out = [f"UPDATE checkout_coupon_reservation SET expires_at=expires_at-{iv} WHERE purchase_id={int(args.id)}"]
        if psql("SELECT to_regclass('checkout_stock_reservation') IS NOT NULL", tuples=True).strip() == "t":
            out.append(f"UPDATE checkout_stock_reservation SET expires_at=expires_at-{iv} WHERE purchase_id={int(args.id)}")
        else:
            print("(checkout_stock_reservation does not exist yet; only the coupon reservation is moved)")
        sql = "; ".join(out)
    else:
        sql = {"quote": f"UPDATE checkout_quote SET expires_at=expires_at-{iv}, created_at=created_at-{iv} WHERE id={int(args.id)}",
               "purchase": f"UPDATE purchase SET created_at=created_at-{iv} WHERE id={int(args.id)}",
               "attempt": f"UPDATE payment SET attempt_started_at=attempt_started_at-{iv} WHERE purchase_id={int(args.id)}"}[args.what]
    print(psql(sql).strip())


# ---------------------------------------------------------------- CLI
def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    for n in ("seed", "reset", "ids", "mpd-calls", "mpd-payments", "mpd-reset", "tchk08", "tchk11"):
        sp.add_parser(n)
    p = sp.add_parser("tchk12"); p.add_argument("--expect-disabled", action="store_true")
    sp.add_parser("tchk13")
    sp.add_parser("tchk14")
    sp.add_parser("tchk15")
    sp.add_parser("tchk16")
    p = sp.add_parser("call"); p.add_argument("method"); p.add_argument("path"); p.add_argument("--as", dest="who"); p.add_argument("--json"); p.add_argument("-H", action="append")
    p = sp.add_parser("burst"); p.add_argument("method", nargs="?"); p.add_argument("path", nargs="?"); p.add_argument("--repeat", type=int, default=1)
    p.add_argument("--as", dest="who"); p.add_argument("--json"); p.add_argument("-H", action="append"); p.add_argument("--spec")
    p = sp.add_parser("coupon"); p.add_argument("code"); p.add_argument("type"); p.add_argument("value", type=int); p.add_argument("--tenant", default="T1"); p.add_argument("--field", action="append")
    p = sp.add_parser("mpd"); p.add_argument("mode"); p.add_argument("--amount-cents", type=int)
    p = sp.add_parser("webhook"); p.add_argument("payment_id"); p.add_argument("--request-id"); p.add_argument("--bad", action="store_true")
    p = sp.add_parser("evidence"); p.add_argument("what", nargs="?", default="all", choices=list(EVIDENCE) + ["stock-reservation", "all"]); p.add_argument("id", nargs="?")
    p = sp.add_parser("db"); p.add_argument("sql")
    p = sp.add_parser("sql"); p.add_argument("sql")
    p = sp.add_parser("shift"); p.add_argument("what", choices=["quote", "purchase", "attempt", "reservation"]); p.add_argument("id"); p.add_argument("minutes")
    a = ap.parse_args()
    if a.cmd == "seed": seed(a)
    elif a.cmd == "reset": reset(a)
    elif a.cmd == "tchk08": tchk08(a)
    elif a.cmd == "tchk11": tchk11(a)
    elif a.cmd == "tchk12": tchk12(a)
    elif a.cmd == "tchk13": tchk13(a)
    elif a.cmd == "tchk14": tchk14(a)
    elif a.cmd == "tchk15": tchk15(a)
    elif a.cmd == "tchk16": tchk16(a)
    elif a.cmd == "ids": print(json.dumps(fixtures(), indent=1))
    elif a.cmd == "call": show(api(a.method.upper(), a.path, a.who, a.json, hdrs(a.H)))
    elif a.cmd == "burst":
        reqs = json.loads(a.spec) if a.spec else [dict(method=a.method.upper(), path=a.path, **{"as": a.who}, json=a.json, headers=hdrs(a.H))] * a.repeat
        res, counts = burst(reqs), {}
        for i, (code, body, ms) in enumerate(res):
            counts[code] = counts.get(code, 0) + 1
            print(f"[{i}] HTTP {code} {ms} ms {json.dumps(body, ensure_ascii=False)[:160]}")
        print("summary:", json.dumps(counts))
    elif a.cmd == "coupon":
        body = {"code": a.code, "couponType": a.type, "value": a.value, **{k: json.loads(v) if v[:1] in '[{"0123456789-tfn' else v for k, v in (x.split("=", 1) for x in a.field or [])}}
        show(api("POST", "/coupons", OWNER_OF[a.tenant], body))
    elif a.cmd == "mpd":
        cfg = {"mode": a.mode}
        if a.amount_cents is not None: cfg["amount_cents"] = a.amount_cents
        show(http("POST", "/_mpd/config", cfg, base=MPD))
    elif a.cmd == "mpd-calls": show(http("GET", "/_mpd/calls", base=MPD))
    elif a.cmd == "mpd-payments": show(http("GET", "/_mpd/payments", base=MPD))
    elif a.cmd == "mpd-reset": show(http("DELETE", "/_mpd/calls", base=MPD))
    elif a.cmd == "webhook": webhook(a)
    elif a.cmd == "evidence": evidence(a)
    elif a.cmd == "db": print(psql(a.sql, readonly=True))
    elif a.cmd == "sql": print(psql(a.sql).strip())
    elif a.cmd == "shift": shift(a)


if __name__ == "__main__":
    main()
