import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const libRs = readFileSync(join(root, "src-tauri/src/lib.rs"), "utf8");

describe("contrato WebView (KoTauri / Telegram Web K)", () => {
  it("habilita clipboard access na janela main", () => {
    assert.match(
      libRs,
      /\.enable_clipboard_access\s*\(\s*\)/,
      "WebviewWindowBuilder da main deve chamar enable_clipboard_access() para colar texto/imagem"
    );
  });

  it("habilita media stream no WebKit (Linux)", () => {
    assert.match(
      libRs,
      /set_enable_media_stream\s*\(\s*true\s*\)/,
      "with_webview deve ligar media_stream para mensagens de voz / getUserMedia"
    );
  });

  it("tem bridge nativo para colar imagem do clipboard", () => {
    assert.match(libRs, /kotauri\.internal\/clipboard-image/);
    assert.match(libRs, /clipboard_image::read_clipboard_png/);
    assert.match(libRs, /clipboard_image_bridge_js/);
  });

  it("permite permission_request no WebKit", () => {
    assert.match(libRs, /connect_permission_request/);
    assert.match(libRs, /request\.allow\s*\(\s*\)/);
  });
});
