import { useEffect, useState } from "react";

const TELEGRAM_WEB_K_URL = "https://web.telegram.org/k/";
const AUTO_OPEN_KEY = "kotauri-pwa-auto-open";

type BeforeInstallPromptEvent = Event & {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
};

function isStandaloneDisplay() {
  if (typeof window === "undefined") return false;
  const media = window.matchMedia("(display-mode: standalone)").matches;
  const iosStandalone =
    "standalone" in window.navigator &&
    Boolean((window.navigator as Navigator & { standalone?: boolean }).standalone);
  return media || iosStandalone;
}

function isIosSafari() {
  if (typeof navigator === "undefined") return false;
  const ua = navigator.userAgent;
  const iOS = /iPad|iPhone|iPod/.test(ua) || (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1);
  const webkit = /WebKit/.test(ua);
  const chromium = /CriOS|FxiOS|EdgiOS|OPiOS|Chrome|Android/.test(ua);
  return iOS && webkit && !chromium;
}

export default function WebShell() {
  const [standalone, setStandalone] = useState(false);
  const [installEvent, setInstallEvent] = useState<BeforeInstallPromptEvent | null>(null);
  const [installedHint, setInstalledHint] = useState(false);
  const [autoOpen, setAutoOpen] = useState(true);
  const [iosTip, setIosTip] = useState(false);
  const [countdown, setCountdown] = useState<number | null>(null);

  useEffect(() => {
    setStandalone(isStandaloneDisplay());
    setIosTip(isIosSafari() && !isStandaloneDisplay());
    const stored = localStorage.getItem(AUTO_OPEN_KEY);
    setAutoOpen(stored !== "0");

    const onBip = (event: Event) => {
      event.preventDefault();
      setInstallEvent(event as BeforeInstallPromptEvent);
    };
    window.addEventListener("beforeinstallprompt", onBip);

    const onInstalled = () => {
      setInstallEvent(null);
      setInstalledHint(true);
    };
    window.addEventListener("appinstalled", onInstalled);

    return () => {
      window.removeEventListener("beforeinstallprompt", onBip);
      window.removeEventListener("appinstalled", onInstalled);
    };
  }, []);

  useEffect(() => {
    if (!standalone || !autoOpen) return;
    setCountdown(2);
    const tick = window.setInterval(() => {
      setCountdown((prev) => {
        if (prev === null) return null;
        if (prev <= 1) {
          window.clearInterval(tick);
          window.location.assign(TELEGRAM_WEB_K_URL);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => window.clearInterval(tick);
  }, [standalone, autoOpen]);

  async function handleInstall() {
    if (!installEvent) return;
    await installEvent.prompt();
    const choice = await installEvent.userChoice;
    if (choice.outcome === "accepted") {
      setInstallEvent(null);
      setInstalledHint(true);
    }
  }

  function toggleAutoOpen(next: boolean) {
    setAutoOpen(next);
    localStorage.setItem(AUTO_OPEN_KEY, next ? "1" : "0");
    if (!next) setCountdown(null);
  }

  function cancelAutoOpen() {
    toggleAutoOpen(false);
  }

  return (
    <main className="web-shell" data-testid="web-shell">
      <div className="web-shell-brand">
        <img
          className="web-shell-logo"
          src={`${import.meta.env.BASE_URL}icon-192.png`}
          width={72}
          height={72}
          alt=""
        />
        <h1>KoTauri</h1>
        <p className="web-shell-tagline">Telegram Web em modo app</p>
      </div>

      {standalone && countdown !== null && autoOpen ? (
        <p className="web-shell-countdown" data-testid="auto-open-countdown">
          Abrindo Telegram em {countdown}s…
          <button type="button" className="web-shell-linkish" onClick={cancelAutoOpen}>
            Cancelar
          </button>
        </p>
      ) : null}

      <div className="web-shell-actions">
        <a
          className="web-shell-cta"
          href={TELEGRAM_WEB_K_URL}
          data-testid="open-telegram-web-k"
        >
          Abrir Telegram
        </a>

        {!standalone && installEvent ? (
          <button
            type="button"
            className="web-shell-cta web-shell-cta-secondary"
            data-testid="install-pwa"
            onClick={handleInstall}
          >
            Instalar no celular
          </button>
        ) : null}

        {!standalone && !installEvent && iosTip ? (
          <div className="web-shell-ios-tip" data-testid="ios-install-tip">
            <strong>Instalar no iPhone</strong>
            <ol>
              <li>Toque em Compartilhar</li>
              <li>Escolha “Adicionar à Tela de Início”</li>
            </ol>
          </div>
        ) : null}

        {!standalone && !installEvent && !iosTip ? (
          <p className="web-shell-install-tip" data-testid="browser-install-tip">
            No Chrome/Edge: menu → “Instalar app” ou “Adicionar à tela inicial”.
          </p>
        ) : null}

        {installedHint ? (
          <p className="web-shell-install-tip" data-testid="installed-hint">
            App instalado. Abra pelo ícone na tela inicial para modo tela cheia.
          </p>
        ) : null}
      </div>

      <label className="web-shell-toggle">
        <input
          type="checkbox"
          checked={autoOpen}
          onChange={(e) => toggleAutoOpen(e.target.checked)}
          data-testid="auto-open-toggle"
        />
        <span>Ao abrir o app instalado, ir direto ao Telegram</span>
      </label>

      {standalone ? (
        <p className="web-shell-mode" data-testid="standalone-badge">
          Modo app (tela cheia)
        </p>
      ) : (
        <p className="web-shell-mode">Navegador — instale para esconder a barra do Chrome</p>
      )}
    </main>
  );
}
