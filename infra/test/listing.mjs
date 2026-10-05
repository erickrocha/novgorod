import assert from "node:assert/strict";

const base = process.env.KIT_BASE_URL || "http://localhost:8080";
assert.ok(["localhost", "127.0.0.1"].includes(new URL(base).hostname), "Disposable local E1 only");
const tokens = new Map();
const identities = {
  s1: ["s1@test.local", "Owner-test-1"],
  s2: ["s2@test.local", "Owner-test-2"],
  u1: ["u1@test.local", "Owner-test-1"],
  c1: ["c1@test.local", "Cust-test-1"],
  sysadmin: ["sysadmin@test.local", process.env.TEST_SYSADMIN_PASSWORD || "SysAdmin-test-1"],
};

async function request(method, path, who, body) {
  const headers = { "Content-Type": "application/json" };
  if (who) {
    if (!tokens.has(who)) {
      const [email, password] = identities[who];
      const login = await request("POST", "/login", null, { email, password });
      assert.equal(login.status, 200, `login ${who}`);
      tokens.set(who, login.body.accessToken || login.body.access_token);
    }
    headers.Authorization = `Bearer ${tokens.get(who)}`;
  }
  if (path === "/login") headers["Content-Type"] = "application/x-www-form-urlencoded";
  const response = await fetch(new URL(path, base), {
    method, headers, body: body === undefined ? undefined :
      path === "/login" ? new URLSearchParams(body).toString() : JSON.stringify(body),
  });
  const text = await response.text();
  return { status: response.status, body: text ? JSON.parse(text) : null };
}

let passed = 0;
let failed = 0;
async function test(name, run) {
  try { await run(); passed++; console.log(`PASS ${name}`); }
  catch (error) { failed++; console.error(`FAIL ${name}: ${error.message}`); }
}

const path = "/tenant/1/listing";
const original = await request("GET", path, "sysadmin");
assert.equal(original.status, 200, "Run ./kit.py seed in disposable E1 first");

try {
  await test("owner/admin write-read persistence and general-field isolation", async () => {
    const before = await request("GET", "/tenant/1", "sysadmin");
    assert.equal(before.status, 200);
    for (const who of ["s1", "sysadmin"]) {
      for (const listed of [true, false]) {
        assert.deepEqual(await request("PUT", path, who, { listed }), { status: 200, body: { listed } });
        assert.deepEqual(await request("GET", path, who), { status: 200, body: { listed } });
      }
    }
    assert.deepEqual(await request("GET", "/tenant/1", "sysadmin"), before);
  });

  await test("cross-tenant and staff authorization without writes", async () => {
    const before = await request("GET", path, "sysadmin");
    for (const [who, status] of [["s2", 404], ["u1", 403], ["c1", 404]]) {
      assert.equal((await request("GET", path, who)).status, status);
      assert.equal((await request("PUT", path, who, { listed: !before.body.listed })).status, status);
    }
    assert.deepEqual(await request("GET", path, "sysadmin"), before);
  });

  await test("missing tenant and malformed write", async () => {
    for (const method of ["GET", "PUT"]) {
      assert.equal((await request(method, "/tenant/999999/listing", "sysadmin", method === "PUT" ? { listed: true } : undefined)).status, 404);
    }
    assert.equal((await request("PUT", path, "s1", { listed: "yes" })).status, 400);
  });

  await test("marketplace catalog does not depend on mobile listing", async () => {
    await request("PUT", path, "s1", { listed: true });
    const before = await request("GET", "/api/public/products", null);
    assert.equal(before.status, 200);
    await request("PUT", path, "s1", { listed: false });
    assert.deepEqual(await request("GET", "/api/public/products", null), before);
  });

  await test("missing Authorization follows shared middleware and returns 403", async () => {
    for (const method of ["GET", "PUT"]) {
      assert.equal((await request(method, path, null, method === "PUT" ? { listed: true } : undefined)).status, 403);
    }
  });
} finally {
  const restored = await request("PUT", path, "sysadmin", original.body);
  assert.equal(restored.status, 200, "restore original listing");
}

console.log(JSON.stringify({ passed, failed }));
process.exitCode = failed ? 1 : 0;
