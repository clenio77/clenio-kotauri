import puppeteer from "puppeteer";

/**
 * Chromium com permissões de clipboard e microfone para validar entrada.
 */
export async function launchBrowser() {
  const browser = await puppeteer.launch({
    headless: true,
    args: [
      "--no-sandbox",
      "--disable-setuid-sandbox",
      "--use-fake-ui-for-media-stream",
      "--use-fake-device-for-media-stream",
      "--autoplay-policy=no-user-gesture-required",
    ],
  });
  return browser;
}

export async function newPage(browser, { origin } = {}) {
  const page = await browser.newPage();
  page.setDefaultTimeout(15_000);

  if (origin) {
    const ctx = browser.defaultBrowserContext();
    await ctx.overridePermissions(origin, [
      "clipboard-read",
      "clipboard-write",
      "microphone",
      "camera",
    ]);

    // Headless às vezes ignora overridePermissions do clipboard; CDP reforça.
    const session = await page.createCDPSession();
    await session.send("Browser.grantPermissions", {
      origin,
      permissions: [
        "clipboardReadWrite",
        "clipboardSanitizedWrite",
        "audioCapture",
        "videoCapture",
      ],
    });
  }

  return page;
}
