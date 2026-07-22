# KoTauri

Cliente desktop leve para Telegram, baseado em **Tauri 2** e na interface **Telegram Web K** (`web.telegram.org/k`). O aplicativo envolve o cliente web oficial do Telegram em uma janela nativa, com painel de configurações em React e personalização por injeção de CSS e JavaScript na sessão web.

<p align="center">
  <img src="public/icon.svg" width="128" height="128" alt="KoTauri">
</p>

## Visão geral

O **KoTauri** separa duas superfícies: uma janela principal em WebView que carrega o Telegram Web K, e uma janela de configurações (Vite + React) servida pelo próprio app. Preferências ficam em JSON no diretório de configuração do usuário; alterações disparam nova injeção de estilos e scripts na janela principal. Há ícone na bandeja do sistema com atalhos para exibir a janela principal, abrir configurações e encerrar o app.

## Funcionalidades principais

- **Telegram Web K embutido**: janela principal apontando para `https://web.telegram.org/k/`, com script inicial que expõe `window.kotauri.openSettings` e navegação interceptada para abrir as configurações locais.
- **Painel de configurações**: UI React na janela “KoTauri Settings” (oculta por padrão); comandos Tauri `get_settings`, `update_setting` e `open_settings`.
- **Injeção dinâmica**: CSS e JS aplicados na página carregada (temas como Midnight/Nord/Catppuccin, modo compacto, fonte e tamanho, bolhas adaptáveis, altura de stickers, barra lateral de pastas estilo Kotatogram, exibição opcional de ID do chat, entre outras opções definidas em `settings.rs` / painel).
- **Bandeja e comportamento de janela**: menu da bandeja (mostrar Telegram, configurações, sair); botão ⚙ flutuante na WebView; opção de minimizar para bandeja em vez de fechar; opção de iniciar minimizado (Linux: permissões de mídia tratadas no WebKit quando aplicável).
- **Downloads nativos**: arquivos baixados pelo Telegram Web K vão para a pasta Downloads do sistema (com nomes únicos se já existirem).
- **Integração**: `tauri-plugin-shell` para abrir URLs no sistema.

## Stack

| Camada | Tecnologias |
|--------|-------------|
| Shell / WebView | Rust, **Tauri 2**, recurso `tray-icon`, **tauri-plugin-shell** |
| Frontend (configurações) | **React 19**, TypeScript, **Vite 6** |
| Persistência | JSON (`serde` / `serde_json`), diretório `kotauri` sob o config do usuário |
| Tooling | `@tauri-apps/cli` ^2, TypeScript ~5.8 |

Versão do projeto: **0.1.3** (veja `package.json` e `src-tauri/tauri.conf.json`).

## Pré-requisitos

- **Node.js** (recomendado: versão **20**, como no CI) e npm.
- **Rust** estável (`rustup`), com `cargo` no PATH.
- **Linux (desenvolvimento e empacotamento .deb)**: dependências de desenvolvimento WebKit/GTK e indicador de app, por exemplo as instaladas no CI: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`, `build-essential`, `curl`, `wget`, `file`. Em runtime, o pacote `.deb` declara dependências como `libwebkit2gtk-4.1-0` e `libgtk-3-0` (ver `tauri.conf.json`).

Consulte a [documentação do Tauri](https://v2.tauri.app/) para pré-requisitos específicos por SO.

## Desenvolvimento

```bash
npm install
npm run tauri dev
```

Isso sobe o Vite na URL configurada em `tauri.conf.json` (`devUrl`, porta **1420**) e inicia o binário Tauri em modo desenvolvimento.

Scripts úteis no `package.json`:

- `npm run dev` — apenas o servidor Vite (frontend).
- `npm run build` — `tsc` + build de produção Vite (`dist/`).
- `npm run tauri` — CLI do Tauri (ex.: `npm run tauri build`).

## PWA (mobile/web)

O frontend também pode rodar como app web instalável (PWA), sem alterar o fluxo desktop em Tauri.

```bash
npm install
npm run dev
```

- Abra a URL do Vite (ou [GitHub Pages](https://clenio77.github.io/clenio-kotauri/)) no celular.
- Toque em **Abrir Telegram** para ir a `https://web.telegram.org/k/`.
- Use **Instalar no celular** (Chrome/Edge) ou “Adicionar à Tela de Início” (iOS) para modo tela cheia.
- No app instalado, o shell pode abrir o Telegram automaticamente (opção na tela inicial).
- Build de produção do PWA (arquivos em `dist/`): `npm run build`.

## Build

```bash
npm run tauri build
```

O fluxo executa `beforeBuildCommand` (`npm run build`) e gera artefatos em `src-tauri/target/release/` e pacotes conforme **bundle.targets** em `tauri.conf.json`.

**Alvos de bundle configurados:** `deb`, `rpm`, `nsis`, `msi`, `dmg`, `app` (macOS). **AppImage não está na lista `targets`**; existe apenas configuração opcional sob `bundle.linux.appimage` — para gerar AppImage é preciso incluir o alvo explicitamente ou ajustar a configuração antes do build.

## Testes e CI

### E2E (Puppeteer)

Valida o shell web, o painel de settings (com mock Tauri) e o contrato de entrada (colar texto/imagem, anexar arquivo, microfone), além de checagens estáticas em `lib.rs` (clipboard + media stream).

```bash
npm run test:e2e
```

Se o frontend já estiver buildado: `npm run test:e2e:run`.

### Pipeline

O pipeline em [`.github/workflows/ci.yml`](.github/workflows/ci.yml) roda em **push** e **pull_request** para `main` e `master`:

1. Instala dependências de sistema (Ubuntu) para Tauri/WebKit.
2. `npm ci` e `npm run build` (TypeScript + Vite).
3. `npm run test:e2e:run` (Puppeteer).
4. `cargo clippy` no `src-tauri` com `-D warnings`.
5. `cargo test` no `src-tauri`.

A construção completa de instaladores (`.deb`, AppImage, etc.) não faz parte do job padrão; o comentário no workflow orienta executar `npm run tauri build` localmente ou em job de release quando for publicar binários.

## Estrutura de pastas

```
.
├── src/                 # Frontend React (janela de configurações)
│   ├── components/
│   └── styles/
├── src-tauri/           # Projeto Rust Tauri (WebView principal, tray, comandos, settings)
├── public/              # Assets estáticos (ex.: ícone)
├── index.html           # Entrada Vite
├── vite.config.ts
├── package.json
└── .github/workflows/   # CI
```

A pasta **`_reference/`** está listada no `.gitignore` como referência técnica opcional (clone mais leve); mantenha cópias locais apenas se precisar de material de consulta.

## Nota legal (Telegram)

Este projeto é um **cliente independente** que exibe o **Telegram Web K** dentro de um contêiner desktop. **Não é produto oficial**, não é endossado pelo Telegram e não representa a Telegram Messenger LLP. Marcas e serviços Telegram pertencem aos respectivos titulares. O uso está sujeito aos termos do Telegram e à legislação aplicável.

## Autor

**Clenio** — ver também `authors` em `src-tauri/Cargo.toml`.
