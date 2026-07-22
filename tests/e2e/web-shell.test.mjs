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

  it("renderiza o shell web", async () => {
    const shell = await page.waitForSelector('[data-testid="web-shell"]');
    assert.ok(shell);
    const title = await page.$eval("h1", (el) => el.textContent?.trim());
    assert.equal(title, "KoTauri Web");
  });

  it("CTA aponta para Telegram Web K", async () => {
    const href = await page.$eval(
      '[data-testid="open-telegram-web-k"]',
      (el) => el.getAttribute("href")
    );
    assert.equal(href, "https://web.telegram.org/k/");
  });

  it("não mostra o painel de settings sem runtime Tauri", async () => {
    const panel = await page.$('[data-testid="settings-panel"]');
    assert.equal(panel, null);
  });
});
