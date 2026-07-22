import assert from "node:assert/strict";
import { after, before, describe, it } from "node:test";
import { APP_ORIGIN } from "../helpers/defaults.mjs";
import { launchBrowser, newPage } from "../helpers/browser.mjs";

describe("web shell (modo sem Tauri)", () => {
  /** @type {import('puppeteer').Browser} */
  let browser;
  /** @type {import('puppeteer').Page} */
  let page;

  before(async () => {
    browser = await launchBrowser();
    page = await newPage(browser);
    await page.goto(APP_ORIGIN, { waitUntil: "networkidle0" });
  });

  after(async () => {
    await browser?.close();
  });

  it("renderiza o shell web app-like", async () => {
    const shell = await page.waitForSelector('[data-testid="web-shell"]');
    assert.ok(shell);
    const title = await page.$eval("h1", (el) => el.textContent?.trim());
    assert.equal(title, "KoTauri");
    const tagline = await page.$eval(".web-shell-tagline", (el) =>
      el.textContent?.trim()
    );
    assert.match(tagline ?? "", /modo app/i);
  });

  it("CTA aponta para Telegram Web K", async () => {
    const href = await page.$eval(
      '[data-testid="open-telegram-web-k"]',
      (el) => el.getAttribute("href")
    );
    assert.equal(href, "https://web.telegram.org/k/");
  });

  it("oferece instalação ou dica no navegador", async () => {
    const installBtn = await page.$('[data-testid="install-pwa"]');
    const browserTip = await page.$('[data-testid="browser-install-tip"]');
    const iosTip = await page.$('[data-testid="ios-install-tip"]');
    assert.ok(installBtn || browserTip || iosTip);
    const autoOpen = await page.$('[data-testid="auto-open-toggle"]');
    assert.ok(autoOpen);
  });

  it("não mostra o painel de settings sem runtime Tauri", async () => {
    const panel = await page.$('[data-testid="settings-panel"]');
    assert.equal(panel, null);
  });
});
