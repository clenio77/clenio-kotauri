#!/usr/bin/env node
/**
 * Sobe preview do app + fixtures e executa a suíte Puppeteer (node:test).
 */
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { readFileSync, existsSync } from "node:fs";
import { createServer as createNetServer } from "node:net";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const fixturesDir = join(root, "tests/fixtures");

async function freePort() {
  return new Promise((resolve, reject) => {
    const srv = createNetServer();
    srv.listen(0, "127.0.0.1", () => {
      const addr = srv.address();
      const port = typeof addr === "object" && addr ? addr.port : 0;
      srv.close((err) => (err ? reject(err) : resolve(port)));
    });
    srv.on("error", reject);
  });
}

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".json": "application/json",
};

function startFixturesServer(port) {
  const server = createServer((req, res) => {
    const url = new URL(req.url || "/", `http://127.0.0.1:${port}`);
    let path = url.pathname === "/" ? "/input-harness.html" : url.pathname;
    path = path.replace(/\.\./g, "");
    const filePath = join(fixturesDir, path);
    if (!filePath.startsWith(fixturesDir) || !existsSync(filePath)) {
      res.writeHead(404);
      res.end("not found");
      return;
    }
    const body = readFileSync(filePath);
    res.writeHead(200, {
      "Content-Type": MIME[extname(filePath)] || "application/octet-stream",
    });
    res.end(body);
  });

  return new Promise((resolve, reject) => {
    server.listen(port, "127.0.0.1", () => resolve(server));
    server.on("error", reject);
  });
}

function waitForUrl(url, attempts = 60) {
  return new Promise(async (resolve, reject) => {
    for (let i = 0; i < attempts; i++) {
      try {
        const res = await fetch(url);
        if (res.ok || res.status === 404) {
          resolve();
          return;
        }
      } catch {
        // retry
      }
      await new Promise((r) => setTimeout(r, 500));
    }
    reject(new Error(`timeout waiting for ${url}`));
  });
}

function startPreview(port) {
  const child = spawn(
    "npx",
    ["vite", "preview", "--host", "127.0.0.1", "--port", String(port)],
    {
      cwd: root,
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env },
    }
  );
  child.stdout.on("data", (d) => process.stdout.write(`[preview] ${d}`));
  child.stderr.on("data", (d) => process.stderr.write(`[preview] ${d}`));
  return child;
}

function runTests(appPort, fixturesPort) {
  const files = [
    "tests/e2e/webview-contract.test.mjs",
    "tests/e2e/web-shell.test.mjs",
    "tests/e2e/settings-panel.test.mjs",
    "tests/e2e/input-capabilities.test.mjs",
  ];

  return new Promise((resolve) => {
    const child = spawn(
      process.execPath,
      ["--test", "--test-concurrency=1", ...files],
      {
        cwd: root,
        stdio: "inherit",
        env: {
          ...process.env,
          KOTAURI_E2E_ORIGIN: `http://127.0.0.1:${appPort}`,
          KOTAURI_FIXTURES_ORIGIN: `http://127.0.0.1:${fixturesPort}`,
        },
      }
    );
    child.on("close", (code) => resolve(code ?? 1));
  });
}

async function main() {
  if (!existsSync(join(root, "dist/index.html"))) {
    console.error("dist/ não encontrado. Rode: npm run build");
    process.exit(1);
  }

  const appPort = Number(process.env.KOTAURI_E2E_PORT) || (await freePort());
  const fixturesPort =
    Number(process.env.KOTAURI_FIXTURES_PORT) || (await freePort());

  const fixtures = await startFixturesServer(fixturesPort);
  const preview = startPreview(appPort);

  let code = 1;
  try {
    await waitForUrl(`http://127.0.0.1:${appPort}/`);
    await waitForUrl(`http://127.0.0.1:${fixturesPort}/input-harness.html`);
    code = await runTests(appPort, fixturesPort);
  } catch (err) {
    console.error(err);
    code = 1;
  } finally {
    preview.kill("SIGTERM");
    fixtures.close();
  }
  process.exit(code);
}

main();
