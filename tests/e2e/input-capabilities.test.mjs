import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { after, before, describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { FIXTURES_ORIGIN } from "../helpers/defaults.mjs";
import { launchBrowser, newPage } from "../helpers/browser.mjs";

const png1x1 = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
  "base64"
);

describe("capacidades de entrada (texto, imagem, áudio)", () => {
  /** @type {import('puppeteer').Browser} */
  let browser;
  /** @type {import('puppeteer').Page} */
  let page;

  before(async () => {
    browser = await launchBrowser();
    page = await newPage(browser, { origin: FIXTURES_ORIGIN });
    await page.goto(`${FIXTURES_ORIGIN}/input-harness.html`, {
      waitUntil: "networkidle0",
    });
    await page.waitForSelector('[data-testid="composer"]');
    // Garante foco + permissões após navegação (clipboard headless).
    await page.bringToFront();
    await page.click('[data-testid="composer"]');
    const session = await page.createCDPSession();
    await session.send("Browser.grantPermissions", {
      origin: FIXTURES_ORIGIN,
      permissions: [
        "clipboardReadWrite",
        "clipboardSanitizedWrite",
        "audioCapture",
        "videoCapture",
      ],
    });
  });

  after(async () => {
    await browser?.close();
  });

  it("expõe Clipboard API e getUserMedia em contexto seguro", async () => {
    const probe = await page.evaluate(() => window.__harness.lastStatus);
    assert.equal(probe.kind, "probe");
    assert.equal(probe.ok, true);
    assert.equal(probe.clipboard, true);
    assert.equal(probe.mediaDevices, true);
    assert.equal(probe.isSecureContext, true);
  });

  it("cola texto de outro lugar via Clipboard API", async () => {
    const sample = "texto externo kotauri-e2e";
    // Preferir Clipboard API real; fallback para buffer de teste (headless).
    await page.evaluate(async (text) => {
      try {
        await navigator.clipboard.writeText(text);
        window.__testClipboard.text = null;
      } catch {
        window.__testClipboard.text = text;
      }
    }, sample);

    await page.click('[data-testid="paste-text"]');
    await page.waitForFunction(
      () => window.__harness?.lastStatus?.kind === "paste-text",
      { timeout: 5000 }
    );

    const status = await page.evaluate(() => window.__harness.lastStatus);
    assert.equal(status.ok, true, String(status.error || ""));
    assert.equal(status.text, sample);

    const composerText = await page.$eval(
      '[data-testid="composer"]',
      (el) => el.innerText.trim()
    );
    assert.match(composerText, /texto externo kotauri-e2e/);
  });

  it("cola imagem do clipboard", async () => {
    await page.evaluate(async (b64) => {
      const binary = atob(b64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      const blob = new Blob([bytes], { type: "image/png" });
      try {
        await navigator.clipboard.write([
          new ClipboardItem({ "image/png": blob }),
        ]);
        window.__testClipboard.imageBlob = null;
      } catch {
        window.__testClipboard.imageBlob = blob;
      }
    }, png1x1.toString("base64"));

    await page.click('[data-testid="paste-image"]');
    await page.waitForFunction(
      () => window.__harness?.lastStatus?.kind === "paste-image",
      { timeout: 5000 }
    );

    const status = await page.evaluate(() => window.__harness.lastStatus);
    assert.equal(status.ok, true, String(status.error || ""));
    assert.ok(status.size > 0);

    const hasImg = await page.$('[data-testid="attached-image"]');
    assert.ok(hasImg);
  });

  it("aceita imagem anexada por file input", async () => {
    const tmpPath = join(
      dirname(fileURLToPath(import.meta.url)),
      "../fixtures/tiny.png"
    );
    writeFileSync(tmpPath, png1x1);

    const input = await page.$('[data-testid="file-input"]');
    assert.ok(input);
    await input.uploadFile(tmpPath);

    await page.waitForFunction(
      () => window.__harness?.lastStatus?.kind === "file",
      { timeout: 5000 }
    );
    const status = await page.evaluate(() => window.__harness.lastStatus);
    assert.equal(status.ok, true);
    assert.match(status.type, /^image\//);
  });

  it("inicia microfone (getUserMedia audio)", async () => {
    await page.click('[data-testid="start-mic"]');
    await page.waitForFunction(
      () => window.__harness?.lastStatus?.kind === "mic",
      { timeout: 8000 }
    );
    const status = await page.evaluate(() => window.__harness.lastStatus);
    assert.equal(status.ok, true, String(status.error || ""));
    assert.ok(Array.isArray(status.tracks));
    assert.ok(status.tracks.length >= 1);

    await page.click('[data-testid="stop-mic"]');
  });
});
