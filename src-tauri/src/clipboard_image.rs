//! Leitura de imagem do clipboard do sistema (GTK) — WebKitGTK muitas vezes
//! entrega texto via Clipboard API, mas não PNG/JPEG ao colar no Telegram Web K.

#[cfg(target_os = "linux")]
pub fn read_clipboard_png() -> Option<Vec<u8>> {
    use gdk_pixbuf::Pixbuf;
    use gtk::gdk;
    use gtk::Clipboard;

    let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
    let pixbuf: Pixbuf = clipboard.wait_for_image()?;
    match pixbuf.save_to_bufferv("png", &[]) {
        Ok(bytes) if !bytes.is_empty() => Some(bytes),
        Ok(_) => None,
        Err(err) => {
            eprintln!("[KoTauri] falha ao codificar clipboard PNG: {err}");
            None
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn read_clipboard_png() -> Option<Vec<u8>> {
    None
}

/// JS injetado na WebView: se o paste do browser não trouxer imagem, pede ao
/// Rust via bridge `kotauri.internal` (mesmo padrão do open-settings).
pub fn clipboard_image_bridge_js() -> &'static str {
    r#"
(function () {
  if (window.__kotauriClipboardBridgeInstalled) return;
  window.__kotauriClipboardBridgeInstalled = true;

  function triggerNativeClipboardImage() {
    try {
      var now = Date.now();
      if (window.__kotauriClipboardLastReq && now - window.__kotauriClipboardLastReq < 600) {
        return;
      }
      window.__kotauriClipboardLastReq = now;
      var f = document.createElement('iframe');
      f.style.display = 'none';
      f.src = 'https://kotauri.internal/clipboard-image';
      document.body.appendChild(f);
      setTimeout(function () { try { f.remove(); } catch (e) {} }, 800);
    } catch (e) {
      console.warn('[KoTauri] clipboard-image bridge failed', e);
    }
  }

  async function browserClipboardHasImage() {
    if (!navigator.clipboard || !navigator.clipboard.read) return false;
    try {
      var items = await navigator.clipboard.read();
      for (var i = 0; i < items.length; i++) {
        var types = items[i].types || [];
        for (var j = 0; j < types.length; j++) {
          if (String(types[j]).indexOf('image/') === 0) return true;
        }
      }
    } catch (e) {
      return false;
    }
    return false;
  }

  function eventHasImage(e) {
    var cd = e && e.clipboardData;
    if (!cd) return false;
    var items = cd.items;
    if (items) {
      for (var i = 0; i < items.length; i++) {
        if (items[i].type && items[i].type.indexOf('image/') === 0) return true;
      }
    }
    var files = cd.files;
    if (files) {
      for (var k = 0; k < files.length; k++) {
        if (files[k].type && files[k].type.indexOf('image/') === 0) return true;
      }
    }
    return false;
  }

  function eventHasText(e) {
    var cd = e && e.clipboardData;
    if (!cd) return false;
    try {
      var t = cd.getData('text/plain');
      return !!(t && String(t).length > 0);
    } catch (err) {
      return false;
    }
  }

  document.addEventListener(
    'paste',
    function (e) {
      if (eventHasImage(e)) return;
      if (eventHasText(e)) return;
      // Paste "vazio" no WebKit: costuma ser imagem só no clipboard do SO.
      triggerNativeClipboardImage();
    },
    true
  );

  document.addEventListener(
    'keydown',
    function (e) {
      var key = e.key || '';
      if (!(e.ctrlKey || e.metaKey)) return;
      if (key !== 'v' && key !== 'V') return;
      // Em paralelo ao paste: se a Clipboard API do browser não vir imagem, usa GTK.
      browserClipboardHasImage().then(function (has) {
        if (!has) triggerNativeClipboardImage();
      });
    },
    true
  );

  window.kotauri = window.kotauri || {};
  window.kotauri.pasteClipboardImage = triggerNativeClipboardImage;

  window.__kotauriDeliverClipboardImage = function (b64, mime) {
    try {
      mime = mime || 'image/png';
      var binary = atob(b64);
      var bytes = new Uint8Array(binary.length);
      for (var i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      var blob = new Blob([bytes], { type: mime });
      var file = new File([blob], 'clipboard.png', { type: mime });
      var dt = new DataTransfer();
      dt.items.add(file);

      function dispatchPaste(target) {
        if (!target) return false;
        // WebKit: ClipboardEvent() ignora clipboardData — define na Event base.
        var evt = new Event('paste', { bubbles: true, cancelable: true });
        try {
          Object.defineProperty(evt, 'clipboardData', {
            get: function () { return dt; }
          });
        } catch (errDef) {
          try {
            evt = new ClipboardEvent('paste', {
              bubbles: true,
              cancelable: true,
              clipboardData: dt
            });
          } catch (errClip) {
            return false;
          }
        }
        target.dispatchEvent(evt);
        return evt.defaultPrevented;
      }

      function dispatchDrop(target) {
        if (!target) return false;
        try {
          target.dispatchEvent(
            new DragEvent('dragenter', { bubbles: true, cancelable: true, dataTransfer: dt })
          );
          target.dispatchEvent(
            new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: dt })
          );
          var dropEvt = new DragEvent('drop', {
            bubbles: true,
            cancelable: true,
            dataTransfer: dt
          });
          target.dispatchEvent(dropEvt);
          return dropEvt.defaultPrevented;
        } catch (errDrop) {
          return false;
        }
      }

      var delivered = false;
      var composer =
        document.querySelector('.input-message-input') ||
        document.querySelector('[contenteditable="true"]') ||
        document.activeElement;

      if (dispatchPaste(composer)) delivered = true;
      if (!delivered && dispatchDrop(composer)) delivered = true;

      var zoneSelectors = [
        '.input-message-container',
        '.chat-input-container',
        '.bubbles',
        '#column-center',
        'body'
      ];
      for (var z = 0; z < zoneSelectors.length && !delivered; z++) {
        var zone = document.querySelector(zoneSelectors[z]);
        if (dispatchPaste(zone)) delivered = true;
        else if (dispatchDrop(zone)) delivered = true;
      }

      var inputs = document.querySelectorAll('input[type="file"]');
      for (var n = 0; n < inputs.length; n++) {
        var input = inputs[n];
        var accept = (input.accept || '').toLowerCase();
        if (
          accept &&
          accept.indexOf('image') === -1 &&
          accept.indexOf('*/*') === -1 &&
          accept !== ''
        ) {
          continue;
        }
        try {
          input.files = dt.files;
          input.dispatchEvent(new Event('input', { bubbles: true }));
          input.dispatchEvent(new Event('change', { bubbles: true }));
          delivered = true;
          break;
        } catch (errInput) {}
      }

      if (!delivered) {
        try {
          var tmp = document.createElement('input');
          tmp.type = 'file';
          tmp.accept = 'image/*';
          tmp.style.display = 'none';
          document.body.appendChild(tmp);
          tmp.files = dt.files;
          tmp.dispatchEvent(new Event('change', { bubbles: true }));
          setTimeout(function () { try { tmp.remove(); } catch (e) {} }, 1000);
          delivered = true;
        } catch (errTmp) {}
      }

      if (!delivered) {
        console.warn('[KoTauri] imagem do clipboard lida, mas não houve handler no Web K');
      } else {
        console.info('[KoTauri] imagem do clipboard entregue ao Web K');
      }
    } catch (err) {
      console.error('[KoTauri] deliver clipboard image failed', err);
    }
  };
})();
"#
}

/// Avalia na WebView o payload PNG em base64.
pub fn deliver_clipboard_image_js(png_base64: &str) -> String {
    format!(
        r#"(function(){{
            if (typeof window.__kotauriDeliverClipboardImage === 'function') {{
                window.__kotauriDeliverClipboardImage({b64}, 'image/png');
            }} else {{
                console.warn('[KoTauri] deliver helper ausente — reinicie o app');
            }}
        }})();"#,
        b64 = serde_json::Value::String(png_base64.to_string())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_js_mentions_internal_route() {
        assert!(clipboard_image_bridge_js().contains("kotauri.internal/clipboard-image"));
        assert!(clipboard_image_bridge_js().contains("__kotauriDeliverClipboardImage"));
    }

    #[test]
    fn deliver_js_embeds_payload() {
        let js = deliver_clipboard_image_js("QUJD");
        assert!(js.contains("QUJD"));
        assert!(js.contains("__kotauriDeliverClipboardImage"));
    }
}
