use std::{env, error::Error};

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use url::Url;
use wry::{http::Request, NewWindowResponse, WebViewBuilder};

const DEFAULT_URL: &str = "https://mariohdel14nasa-lang.github.io/essentialservicesweb/";
const TARGET_OS: &str = if cfg!(target_os = "macos") {
    "macOS"
} else if cfg!(target_os = "windows") {
    "Windows"
} else if cfg!(target_os = "linux") {
    "Linux"
} else {
    "Unknown"
};

const WINDOW_BUTTONS_SCRIPT: &str = r#"
(() => {
  if (window.top !== window || window.__essentialServicesControlsInstalled) return;
  window.__essentialServicesControlsInstalled = true;
  const targetOS = "__TARGET_OS__";

  const host = document.createElement("div");
  host.id = "__essential_services_window_controls";
  host.style.cssText = "all:initial!important;display:block!important;visibility:visible!important;opacity:1!important;position:fixed!important;inset:0!important;z-index:2147483647!important;pointer-events:none!important";

  const root = host.attachShadow({ mode: "closed" });
  root.innerHTML = `
    <style>
      :host { all: initial; }
      .controls {
        position: fixed; top: 20px; left: 50%; transform: translateX(-50%);
        display: flex; align-items: center; gap: 6px; height: 35px; padding: 0 10px;
        box-sizing: border-box;
        border: 1px solid #ffffff44; border-radius: 999px;
        background: #101827e8; box-shadow: 0 4px 16px #0005;
        backdrop-filter: blur(8px); pointer-events: auto; cursor: grab; z-index: 3;
        visibility: visible; opacity: 1;
      }
      .controls button {
        width: 20px; height: 20px; padding: 0; display: grid; place-items: center;
        flex: 0 0 20px;
        border: 1px solid #ffffff44; border-radius: 50%;
        background: #25334a; color: #f3f6fb; cursor: pointer;
        transition: background .15s ease, transform .15s ease;
      }
      .controls button:hover { background: #34445f; transform: scale(1.08); }
      button:focus-visible { outline: 3px solid #75aaff; outline-offset: 3px; }
      .controls button.close:hover { background: #c83d4b; }
      svg { fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
      .controls svg { width: 13px; height: 13px; }
      .sidebar {
        position: fixed; top: 50%; left: 18px; transform: translateY(-50%);
        display: flex; flex-direction: column; align-items: flex-start; gap: 14px;
        pointer-events: auto; z-index: 2;
      }
      .main-nav-group {
        display: flex; flex-direction: column; gap: 5px; padding: 9px;
        border-radius: 28px; background: #6500f5;
        box-shadow: 0 8px 24px #0006;
      }
      .sidebar button {
        min-width: 132px; height: 50px; padding: 0 14px; display: flex;
        align-items: center; gap: 11px; border: 0; border-radius: 999px;
        background: transparent; color: #24113c; cursor: pointer;
        font: 600 14px "Segoe UI", Arial, sans-serif; text-align: left;
        transition: background .15s ease, color .15s ease, transform .15s ease;
      }
      .main-nav-group button:hover { background: #7e2cff; color: #fff; }
      .main-nav-group button[aria-current="page"] { background: #17121e; color: #b98aff; }
      .sidebar button[data-section="settings"] {
        background: #17121e; color: #c9adff;
        box-shadow: 0 8px 24px #0006;
      }
      .sidebar button[data-section="settings"]:hover,
      .sidebar button[data-section="settings"][aria-current="page"] {
        background: #282131; color: #dfcaff;
      }
      .sidebar button:focus-visible { outline: 2px solid #fff; outline-offset: 2px; }
      .sidebar svg { width: 22px; height: 22px; flex: 0 0 22px; fill: currentColor; stroke: none; }
      .panel {
        position: fixed; left: 184px; top: 50%; transform: translateY(-50%);
        width: 280px; max-height: min(70vh, 500px); overflow: auto;
        padding: 20px; border: 1px solid #ffffff24; border-radius: 20px;
        background: #201e25f5; color: #e7e3eb; box-shadow: 0 8px 28px #0006;
        backdrop-filter: blur(12px); pointer-events: auto;
        font: 14px "Segoe UI", Arial, sans-serif;
      }
      .panel[hidden] { display: none; }
      .panel h2 { margin: 0 0 12px; font-size: 18px; }
      .panel p { margin: 0; color: #aebbd0; line-height: 1.5; }
      .apps-panel {
        inset: 0; width: auto; height: 100vh; max-height: none; padding: 86px 7vw 48px 210px;
        box-sizing: border-box; transform: none; border: 0; border-radius: 0;
        background: #201e25; box-shadow: none; overflow: auto; z-index: 1;
      }
      .catalog-content { width: min(100%, 980px); margin: 0 auto; }
      .catalog-heading { display: flex; align-items: center; gap: 18px; margin-bottom: 52px; }
      .catalog-heading h2 { margin: 0; font-size: clamp(26px, 3vw, 40px); font-weight: 500; }
      .catalog-heading p { margin-top: 5px; font-size: 16px; }
      .catalog-heading-icon {
        width: 76px; height: 76px; display: grid; place-items: center;
        border-radius: 50%; background: #17121e; color: #c9adff; flex: 0 0 76px;
      }
      .catalog-heading-icon svg { width: 34px; height: 34px; fill: currentColor; }
      .category-tabs {
        display: flex; flex-wrap: wrap; gap: 8px; margin: 0 0 28px;
        padding: 7px; border-radius: 999px; background: #17121e; width: fit-content;
      }
      .category-tab {
        min-height: 40px; padding: 0 17px; border: 0; border-radius: 999px;
        background: transparent; color: #c6bfce; cursor: pointer;
        font: 600 14px "Segoe UI", Arial, sans-serif;
      }
      .category-tab:hover { background: #33283f; color: #fff; }
      .category-tab[aria-pressed="true"] { background: #6500f5; color: #fff; }
      .app-list { display: flex; flex-direction: column; gap: 18px; }
      .app-card {
        display: grid; grid-template-columns: 76px minmax(0, 1fr) 220px;
        align-items: center; gap: 22px; min-height: 112px; padding: 10px 0;
      }
      .app-icon {
        width: 76px; height: 76px; display: grid; place-items: center;
        border-radius: 22px; background: #30243d; color: #d3b6ff;
        font: 700 25px "Segoe UI", Arial, sans-serif;
      }
      .app-details h3 { margin: 0 0 4px; font-size: 25px; font-weight: 500; }
      .app-details p { margin: 0; color: #c6bfce; font-size: 17px; }
      .app-download {
        min-height: 72px; border: 0; border-radius: 24px;
        background: #6500f5; color: #17121e;
        font: 600 20px "Segoe UI", Arial, sans-serif;
      }
      .app-download:disabled { opacity: .48; cursor: not-allowed; }
      .catalog-empty {
        min-height: 180px; display: grid; place-content: center; gap: 10px;
        border: 1px solid #ffffff14; border-radius: 20px; background: #ffffff06;
        text-align: center;
      }
      .catalog-empty[hidden], .app-list[hidden] { display: none; }
      .catalog-empty strong { font-size: 17px; font-weight: 600; }
      .catalog-empty p { max-width: 430px; margin: 0 auto; color: #bdb8c4; }
      .settings-panel {
        inset: 0; width: auto; height: 100vh; max-height: none; padding: 48px 7vw;
        box-sizing: border-box; transform: none; border: 0; border-radius: 0;
        background: #201e25; box-shadow: none; overflow: auto; z-index: 1;
      }
      .settings-content { width: min(100%, 980px); margin: 0 auto; }
      .settings-content > h1 { margin: 0 0 30px; font-size: 32px; font-weight: 500; }
      .settings-section { margin: 0 0 26px; }
      .settings-section > h2 {
        margin: 0 0 10px; color: #8e36ff; font-size: 14px; font-weight: 600;
      }
      .settings-card {
        overflow: hidden; border-radius: 17px; background: #39373e;
      }
      .setting-row {
        min-height: 76px; padding: 14px 18px; display: flex; align-items: center;
        gap: 16px; color: #e7e3eb; cursor: pointer;
      }
      .setting-row + .setting-row { border-top: 1px solid #ffffff0d; }
      .setting-icon {
        width: 24px; flex: 0 0 24px; color: #8000ff;
        font-size: 21px; text-align: center;
      }
      .setting-copy { flex: 1; min-width: 0; }
      .setting-copy strong { display: block; margin-bottom: 4px; font-size: 14px; font-weight: 500; }
      .setting-copy small { display: block; color: #c6bfce; font-size: 12px; line-height: 1.4; }
      .setting-switch {
        appearance: none; width: 48px; height: 28px; margin: 0; flex: 0 0 48px;
        position: relative; border: 0; border-radius: 999px;
        background: #77747c; cursor: pointer; transition: background .18s ease;
      }
      .setting-switch::after {
        content: ""; position: absolute; top: 4px; left: 4px;
        width: 20px; height: 20px; border-radius: 50%; background: #ddd9e2;
        transition: transform .18s ease, background .18s ease;
      }
      .setting-switch:checked { background: #6500f5; }
      .setting-switch:checked::after { transform: translateX(20px); background: #17121e; }
      .setting-switch:disabled { opacity: .48; cursor: not-allowed; }
      .setting-switch:focus-visible { outline: 2px solid #d3b6ff; outline-offset: 3px; }
      .language-select {
        min-width: 150px; padding: 10px 34px 10px 12px; border: 1px solid #ffffff24;
        border-radius: 10px; background: #28262d; color: #f3f0f6;
        font: 14px "Segoe UI", Arial, sans-serif;
      }
      .language-select:focus-visible { outline: 2px solid #d3b6ff; outline-offset: 2px; }
      .settings-update {
        width: 100%; min-height: 64px; padding: 12px 20px; border: 0;
        border-radius: 18px; background: #448e71; color: #fff;
        text-align: left; cursor: pointer; font: 600 16px "Segoe UI", Arial, sans-serif;
      }
      .settings-update:hover { background: #55a383; }
      .credits-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px 28px; }
      .credit strong { display: block; margin-bottom: 3px; color: #e7e3eb; font-size: 14px; font-weight: 500; }
      .credit span { color: #bdb8c4; font-size: 12px; }
      .about-card { padding: 28px; text-align: center; }
      .about-card h2 { margin: 0 0 12px; font-size: 28px; font-weight: 500; }
      .about-card p { margin: 0 auto 12px; max-width: 620px; color: #d3ced8; line-height: 1.5; }
      .about-links { display: flex; justify-content: center; flex-wrap: wrap; gap: 10px; margin-top: 20px; }
      .about-links a {
        min-height: 42px; padding: 0 18px; display: inline-flex; align-items: center;
        border-radius: 999px; background: #6500f5; color: #17121e;
        font-weight: 600; text-decoration: none;
      }
      .about-links a:hover { background: #8433ff; color: #fff; }
      :host([data-pitch-black="true"]) .apps-panel,
      :host([data-pitch-black="true"]) .settings-panel { background: #000; }
      :host([data-pitch-black="true"]) .settings-card { background: #171717; }
      :host([data-use-blur="false"]) .controls,
      :host([data-use-blur="false"]) .panel { backdrop-filter: none; }
      @media (max-width: 520px) {
        .sidebar { left: 8px; }
        .main-nav-group { padding: 7px; }
        .sidebar button { min-width: 48px; width: 48px; justify-content: center; padding: 0; }
        .sidebar button span { display: none; }
        .panel { left: 72px; width: min(420px, calc(100vw - 84px)); }
        .apps-panel { padding: 76px 20px 28px 72px; }
        .catalog-heading { gap: 12px; margin: 0 0 32px 10px; }
        .catalog-heading-icon { width: 56px; height: 56px; flex-basis: 56px; }
        .catalog-heading-icon svg { width: 26px; height: 26px; }
        .category-tabs { width: 100%; justify-content: center; border-radius: 18px; }
        .category-tab { padding: 0 10px; font-size: 12px; }
        .app-card { grid-template-columns: 52px minmax(0, 1fr); gap: 12px; }
        .app-icon { width: 52px; height: 52px; border-radius: 16px; font-size: 19px; }
        .app-details h3 { font-size: 18px; }
        .app-details p { font-size: 14px; }
        .app-download { grid-column: 2; min-height: 44px; font-size: 15px; }
        .settings-panel { padding: 76px 16px 28px 72px; }
        .settings-content > h1 { font-size: 26px; }
        .setting-row { padding: 12px; gap: 10px; }
        .setting-icon { display: none; }
        .credits-list { grid-template-columns: 1fr; gap: 12px; }
        .about-card { padding: 20px 14px; }
        .about-card h2 { font-size: 23px; }
      }
    </style>
    <div class="controls" aria-label="Controles de ventana" data-i18n-aria="windowControls">
      <button id="minimize" aria-label="Minimizar" title="Minimizar" data-i18n-aria="minimize" data-i18n-title="minimize">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14"/></svg>
      </button>
      <button id="maximize" aria-label="Maximizar" title="Maximizar" data-i18n-aria="maximize" data-i18n-title="maximize">
        <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="5" y="5" width="14" height="14" rx="1"/></svg>
      </button>
      <button class="close" id="close" aria-label="Cerrar" title="Cerrar" data-i18n-aria="close" data-i18n-title="close">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18"/></svg>
      </button>
    </div>
    <nav class="sidebar" aria-label="Menú principal" data-i18n-aria="mainMenu">
      <div class="main-nav-group">
        <button data-section="home" aria-current="page">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5.85,17.1q1.27-.98 2.85-1.54T12,15t3.3,.56t2.85,1.54q.88-1.03 1.36-2.33T20,12q0-3.33-2.34-5.66T12,4Q8.68,4 6.34,6.34T4,12q0,1.47 .49,2.78q.49,1.3 1.36,2.33Zm3.66-5.11Q8.5,10.98 8.5,9.5q0-1.48 1.01-2.49T12,6t2.49,1.01T15.5,9.5t-1.01,2.49T12,13q-1.47,0-2.49-1.01ZM12,22q-2.07,0-3.9-.79q-1.82-.79-3.17-2.14Q3.58,17.73 2.79,15.9T2,12Q2,9.92 2.79,8.1Q3.58,6.27 4.93,4.93Q6.28,3.57 8.1,2.79Q9.93,2 12,2t3.9,.79t3.17,2.14q1.35,1.35 2.14,3.17Q22,9.92 22,12t-.79,3.9q-.79,1.82-2.14,3.17q-1.35,1.35-3.17,2.14T12,22Zm2.5-2.39q1.18-.39 2.15-1.11q-.98-.73-2.15-1.11T12,17t-2.5,.39Q8.33,17.77 7.35,18.5q.98,.73 2.15,1.11T12,20t2.5-.39Zm-1.42-9.04Q13.5,10.15 13.5,9.5T13.08,8.42T12,8t-1.07,.42T10.5,9.5t.43,1.07T12,11t1.08-.43ZM12,9.5Zm0,9Z"/></svg>
          <span data-i18n="home">Home</span>
        </button>
        <button data-section="apps">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4.59,19.41Q4,18.83 4,18t.59-1.41T6,16q.83,0 1.41,.59Q8,17.18 8,18T7.41,19.41T6,20t-1.41-.59Zm6,0Q10,18.83 10,18t.59-1.41T12,16t1.41,.59Q14,17.18 14,18t-.59,1.41T12,20t-1.41-.59Zm6,0Q16,18.83 16,18t.59-1.41T18,16t1.41,.59Q20,17.18 20,18t-.59,1.41T18,20t-1.41-.59Zm-12-6Q4,12.83 4,12t.59-1.41T6,10q.83,0 1.41,.59Q8,11.18 8,12t-.59,1.41T6,14t-1.41-.59Zm6,0Q10,12.83 10,12t.59-1.41T12,10t1.41,.59Q14,11.18 14,12t-.59,1.41T12,14t-1.41-.59Zm6,0Q16,12.83 16,12t.59-1.41T18,10t1.41,.59Q20,11.18 20,12t-.59,1.41T18,14t-1.41-.59Zm-12-6Q4,6.82 4,6T4.59,4.59T6,4q.83,0 1.41,.59Q8,5.18 8,6t-.59,1.41T6,8t-1.41-.59Zm6,0Q10,6.82 10,6t.59-1.41T12,4t1.41,.59Q14,5.18 14,6t-.59,1.41T12,8t-1.41-.59Zm6,0Q16,6.82 16,6t.59-1.41T18,4t1.41,.59Q20,5.18 20,6t-.59,1.41T18,8t-1.41-.59Z"/></svg>
          <span data-i18n="apps">Apps</span>
        </button>
      </div>
      <button data-section="settings">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10.83,22q-0.68,0-1.16-0.45t-0.59-1.1L8.85,18.8q-0.32-0.13-0.61-0.3T7.68,18.13L6.13,18.77Q5.5,19.05 4.88,18.83T3.9,18.02L2.73,15.98q-0.35-0.57-0.2-1.22T3.2,13.68l1.33-1Q4.5,12.5 4.5,12.34q0-0.16 0-0.34t0-0.34t0.03-0.34l-1.33-1Q2.68,9.9 2.53,9.25t0.2-1.23L3.9,5.97Q4.25,5.4 4.88,5.18T6.13,5.22L7.68,5.88Q7.95,5.68 8.25,5.5t0.6-0.3L9.08,3.55q0.1-0.65 0.59-1.1T10.83,2h2.35q0.68 0 1.16,0.45t0.59,1.1L15.15,5.2q0.33,0.13 0.61,0.3t0.56,0.38l1.55-0.65q0.63-0.27 1.25-0.05t0.98,0.8l1.18,2.05q0.35,0.58 0.2,1.23T20.8,10.33l-1.32,1q0.02,0.18 0.02,0.34t0,0.34t0,0.34t-0.05,0.34l1.32,1q0.52,0.43 0.68,1.08t-0.2,1.22l-1.2,2.05q-0.35,0.58-0.98,0.8t-1.25-0.05l-1.5-0.65q-0.27,0.2-0.57,0.38t-0.6,0.3l-0.22,1.65q-0.1,0.65-0.59,1.1T13.18,22H10.83ZM11,20h1.98l0.35-2.65q0.78-0.2 1.44-0.59t1.21-0.94l2.47,1.03l0.98-1.7l-2.15-1.63q0.13-0.35 0.17-0.74T17.5,12t-0.05-0.79q-0.05-0.39-0.17-0.74l2.15-1.63l-0.98-1.7L15.98,8.2q-0.55-0.57-1.21-0.96T13.33,6.65L13,4H11.03L10.68,6.65Q9.9,6.85 9.24,7.24T8.03,8.17L5.55,7.15L4.58,8.85l2.15,1.6Q6.6,10.83 6.55,11.2T6.5,12q0,0.4 0.05,0.77t0.17,0.75L4.58,15.15l0.98,1.7L8.03,15.8q0.55,0.58 1.21,0.96t1.44,0.59L11,20Zm1.05-4.5q1.45,0 2.47-1.03T15.55,12T14.53,9.52T12.05,8.5q-1.47,0-2.49,1.02T8.55,12t1.01,2.47t2.49,1.03ZM12,12Z"/></svg>
        <span data-i18n="settings">Ajustes</span>
      </button>
    </nav>
    <section class="panel apps-panel" id="apps-panel" aria-labelledby="apps-title" hidden>
      <div class="catalog-content">
        <div class="catalog-heading">
          <div class="catalog-heading-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24"><path d="M10 4H3a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h18a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2H12l-2-2z"/></svg>
          </div>
          <div><h2 id="apps-title">Utilidades</h2><p><span id="apps-count"></span> · <span id="apps-platform-label" data-i18n="catalogFor">Catálogo para</span> <span id="apps-platform"></span></p></div>
        </div>
        <nav class="category-tabs" aria-label="Categorías de aplicaciones" data-i18n-aria="appCategories">
          <button class="category-tab" type="button" data-category="music" aria-pressed="false" data-i18n="music">Música</button>
          <button class="category-tab" type="button" data-category="videos" aria-pressed="false" data-i18n="videos">Vídeos y Películas</button>
          <button class="category-tab" type="button" data-category="utilities" aria-pressed="true" data-i18n="utilities">Utilidades</button>
          <button class="category-tab" type="button" data-category="games" aria-pressed="false" data-i18n="games">Juegos</button>
        </nav>
        <div class="catalog-empty" id="catalog-empty" hidden>
          <strong id="catalog-empty-title"></strong>
          <p><span data-i18n="catalogEmptyPrefix">El catálogo de</span> <span id="apps-platform-empty"></span> <span data-i18n="catalogEmptySuffix">aparecerá aquí cuando se añadan aplicaciones.</span></p>
        </div>
        <div class="app-list" id="app-list" aria-live="polite"></div>
      </div>
    </section>
    <section class="panel settings-panel" id="settings-panel" aria-labelledby="settings-title" hidden>
      <div class="settings-content">
        <h1 id="settings-title" data-i18n="settings">Ajustes</h1>
        <section class="settings-section" aria-labelledby="language-heading">
          <h2 id="language-heading" data-i18n="languageHeading">Idioma / Language</h2>
          <div class="settings-card">
            <label class="setting-row" for="setting-language">
              <span class="setting-icon" aria-hidden="true">文</span>
              <span class="setting-copy"><strong data-i18n="languageLabel">Idioma de la aplicación</strong><small data-i18n="languageDescription">Selecciona el idioma de la interfaz.</small></span>
              <select class="language-select" id="setting-language" aria-label="Idioma / Language">
                <option value="es">Español</option>
                <option value="en">English</option>
              </select>
            </label>
          </div>
        </section>
        <section class="settings-section" aria-labelledby="profiles-heading">
          <h2 id="profiles-heading" data-i18n="moreProfiles">Más perfiles</h2>
          <div class="settings-card">
            <label class="setting-row">
              <span class="setting-icon" aria-hidden="true">▦</span>
              <span class="setting-copy"><strong data-i18n="apps">Apps</strong><small data-i18n="allEssentialApps">All Essential Apps</small></span>
              <input class="setting-switch" id="setting-apps-enabled" type="checkbox" role="switch" aria-label="Mostrar Apps" data-i18n-aria="showApps">
            </label>
          </div>
        </section>
        <section class="settings-section" aria-labelledby="general-heading">
          <h2 id="general-heading" data-i18n="general">General</h2>
          <div class="settings-card">
            <label class="setting-row">
              <span class="setting-icon" aria-hidden="true">◉</span>
              <span class="setting-copy">
                <strong data-i18n="shareData">Compartir datos de Essential Account con otras apps</strong>
                <small data-i18n="shareDataDescription">Preferencia guardada. El acceso por URL y deep links aún no está disponible en esta app de escritorio.</small>
              </span>
              <input class="setting-switch" id="setting-share-data" type="checkbox" role="switch" aria-label="Compartir datos de cuenta" data-i18n-aria="shareData">
            </label>
            <label class="setting-row" title="El acceso cuando el dispositivo está bloqueado lo controla el sistema operativo." data-i18n-title="lockedTitle">
              <span class="setting-icon" aria-hidden="true">▣</span>
              <span class="setting-copy">
                <strong data-i18n="allowLocked">Permitir cuando esté bloqueado</strong>
                <small data-i18n="allowLockedDescription">No disponible en escritorio: el bloqueo lo controla el sistema operativo.</small>
              </span>
              <input class="setting-switch" id="setting-allow-locked" type="checkbox" role="switch" aria-label="Permitir cuando esté bloqueado" data-i18n-aria="allowLocked" disabled>
            </label>
          </div>
        </section>
        <button class="settings-update" id="settings-update" type="button" data-i18n="update">↓　Actualizar página de Essential Services</button>
        <section class="settings-section" aria-labelledby="appearance-heading">
          <h2 id="appearance-heading" data-i18n="appearance">Apariencia</h2>
          <div class="settings-card">
            <label class="setting-row">
              <span class="setting-icon" aria-hidden="true">◐</span>
              <span class="setting-copy"><strong data-i18n="pitchBlack">Modo negro puro</strong><small data-i18n="pitchBlackDescription">Usar un fondo negro en las vistas de la aplicación.</small></span>
              <input class="setting-switch" id="setting-pitch-black" type="checkbox" role="switch" aria-label="Modo negro puro" data-i18n-aria="pitchBlack">
            </label>
            <label class="setting-row">
              <span class="setting-icon" aria-hidden="true">≋</span>
              <span class="setting-copy"><strong data-i18n="useBlur">Usar desenfoque</strong><small data-i18n="useBlurDescription">Aplicar desenfoque progresivo a los elementos de la interfaz.</small></span>
              <input class="setting-switch" id="setting-use-blur" type="checkbox" role="switch" aria-label="Usar desenfoque" data-i18n-aria="useBlur">
            </label>
          </div>
        </section>
        <section class="settings-section" aria-labelledby="credits-heading">
          <h2 id="credits-heading" data-i18n="credits">Créditos</h2>
          <div class="credits-list">
            <div class="credit"><strong>fossifyorg</strong><span data-i18n="calculatorCreators">Creadores de Calculator</span></div>
            <div class="credit"><strong>sameerasw</strong><span data-i18n="draftCreator">Creador de Draft y de la app base</span></div>
            <div class="credit"><strong>ivorisnoob</strong><span data-i18n="kodaCreator">Creador de Koda</span></div>
            <div class="credit"><strong>shrimqy</strong><span data-i18n="sefirahCreator">Creador de Sefirah</span></div>
          </div>
        </section>
        <section class="settings-section" aria-labelledby="about-heading">
          <h2 id="about-heading" data-i18n="about">Acerca de</h2>
          <div class="settings-card about-card">
            <h2 id="about-title">Essential Services v1.0</h2>
            <p data-i18n="aboutDescription">A service app for other apps</p>
            <p data-i18n="aboutCredits">Desarrollado por mariohdel14nasa-lang con ❤️ desde 🇪🇸</p>
            <div class="about-links">
              <a href="https://mariohdel14nasa-lang.github.io/essentialservicesweb/" target="_blank" rel="noopener noreferrer" data-i18n="website">Website</a>
              <a href="https://github.com/mariohdel14nasa-lang/essential_services_desktop" target="_blank" rel="noopener noreferrer" data-i18n="viewGithub">Ver en GitHub</a>
            </div>
          </div>
        </section>
      </div>
    </section>`;

  root.querySelector(".controls").addEventListener("pointerdown", event => {
    if (event.button === 0 && !event.target.closest("button")) {
      window.ipc.postMessage("drag");
    }
  });
  const panels = {
    apps: root.getElementById("apps-panel"),
    settings: root.getElementById("settings-panel")
  };
  root.getElementById("apps-platform").textContent = targetOS;
  root.getElementById("apps-platform-empty").textContent = targetOS;
  const preferenceKey = name => `essential-services-desktop.settings.${name}`;
  const readPreference = (name, defaultValue) => {
    const savedValue = localStorage.getItem(preferenceKey(name));
    return savedValue === null ? defaultValue : savedValue === "true";
  };
  const savePreference = (name, value) => localStorage.setItem(preferenceKey(name), String(value));
  const translations = {
    es: {
      windowControls: "Controles de ventana", minimize: "Minimizar", maximize: "Maximizar", close: "Cerrar",
      mainMenu: "Menú principal", home: "Home", apps: "Apps", settings: "Ajustes",
      catalogFor: "Catálogo para", appCategories: "Categorías de aplicaciones",
      music: "Música", videos: "Vídeos y Películas", utilities: "Utilidades", games: "Juegos",
      catalogEmptyPrefix: "El catálogo de", catalogEmptySuffix: "aparecerá aquí cuando se añadan aplicaciones.",
      languageHeading: "Idioma / Language", languageLabel: "Idioma de la aplicación",
      languageDescription: "Selecciona el idioma de la interfaz.",
      moreProfiles: "Más perfiles", allEssentialApps: "Todas las apps de Essential",
      showApps: "Mostrar Apps", general: "General",
      shareData: "Compartir datos de Essential Account con otras apps",
      shareDataDescription: "Preferencia guardada. El acceso por URL y deep links aún no está disponible en esta app de escritorio.",
      allowLocked: "Permitir cuando esté bloqueado",
      allowLockedDescription: "No disponible en escritorio: el bloqueo lo controla el sistema operativo.",
      lockedTitle: "El acceso cuando el dispositivo está bloqueado lo controla el sistema operativo.",
      update: "↓　Actualizar página de Essential Services", appearance: "Apariencia",
      pitchBlack: "Modo negro puro", pitchBlackDescription: "Usar un fondo negro en las vistas de la aplicación.",
      useBlur: "Usar desenfoque", useBlurDescription: "Aplicar desenfoque progresivo a los elementos de la interfaz.",
      credits: "Créditos", calculatorCreators: "Creadores de Calculator",
      draftCreator: "Creador de Draft y de la app base", kodaCreator: "Creador de Koda",
      sefirahCreator: "Creador de Sefirah", about: "Acerca de",
      aboutDescription: "A service app for other apps",
      aboutCredits: "Desarrollado por mariohdel14nasa-lang con ❤️ desde 🇪🇸",
      website: "Website", viewGithub: "Ver en GitHub",
      emptyCategory: "No hay apps de {category} disponibles todavía",
      appCount: "{count} {label}", appSingular: "app", appPlural: "apps",
      pendingDownload: "Descarga pendiente", downloadNotConfigured: "El enlace de descarga aún no está configurado",
      restore: "Restaurar"
    },
    en: {
      windowControls: "Window controls", minimize: "Minimize", maximize: "Maximize", close: "Close",
      mainMenu: "Main menu", home: "Home", apps: "Apps", settings: "Settings",
      catalogFor: "Catalog for", appCategories: "App categories",
      music: "Music", videos: "Videos & Movies", utilities: "Utilities", games: "Games",
      catalogEmptyPrefix: "The", catalogEmptySuffix: "catalog will appear here when apps are added.",
      languageHeading: "Language", languageLabel: "Application language",
      languageDescription: "Choose the interface language.",
      moreProfiles: "More profiles", allEssentialApps: "All Essential Apps",
      showApps: "Show Apps", general: "General",
      shareData: "Share Essential Account data with other apps",
      shareDataDescription: "Preference saved. URL access and deep links are not available in this desktop app yet.",
      allowLocked: "Allow when locked",
      allowLockedDescription: "Unavailable on desktop: screen locking is controlled by the operating system.",
      lockedTitle: "Access while the device is locked is controlled by the operating system.",
      update: "↓　Refresh Essential Services page", appearance: "Appearance",
      pitchBlack: "Pitch Black Mode", pitchBlackDescription: "Use a black background in the application views.",
      useBlur: "Use Blur", useBlurDescription: "Apply progressive blur to interface elements.",
      credits: "Credits", calculatorCreators: "Creators of Calculator",
      draftCreator: "Creator of Draft and the base app", kodaCreator: "Creator of Koda",
      sefirahCreator: "Creator of Sefirah", about: "About",
      aboutDescription: "A service app for other apps",
      aboutCredits: "Developed by mariohdel14nasa-lang with ❤️ from 🇪🇸",
      website: "Website", viewGithub: "View on GitHub",
      emptyCategory: "No {category} apps available yet",
      appCount: "{count} {label}", appSingular: "app", appPlural: "apps",
      pendingDownload: "Download pending", downloadNotConfigured: "The download link has not been configured yet",
      restore: "Restore"
    }
  };
  const language = localStorage.getItem(preferenceKey("language")) === "en" ? "en" : "es";
  const t = (key, values = {}) => {
    let text = translations[language][key] || translations.es[key] || key;
    for (const [name, value] of Object.entries(values)) {
      text = text.replaceAll(`{${name}}`, value);
    }
    return text;
  };
  const applyLanguage = () => {
    root.querySelectorAll("[data-i18n]").forEach(element => {
      element.textContent = t(element.dataset.i18n);
    });
    root.querySelectorAll("[data-i18n-aria]").forEach(element => {
      element.setAttribute("aria-label", t(element.dataset.i18nAria));
    });
    root.querySelectorAll("[data-i18n-title]").forEach(element => {
      element.title = t(element.dataset.i18nTitle);
    });
    root.querySelectorAll("[data-i18n-placeholder]").forEach(element => {
      element.setAttribute("placeholder", t(element.dataset.i18nPlaceholder));
    });
  };
  const languageSelect = root.getElementById("setting-language");
  languageSelect.value = language;
  languageSelect.addEventListener("change", () => {
    localStorage.setItem(preferenceKey("language"), languageSelect.value);
    window.location.reload();
  });
  applyLanguage();
  const settings = {
    appsEnabled: readPreference("appsEnabled", true),
    shareAccountData: readPreference("shareAccountData", true),
    pitchBlack: readPreference("pitchBlack", false),
    useBlur: readPreference("useBlur", true)
  };
  const appsButton = root.querySelector('[data-section="apps"]');
  const homeButton = root.querySelector('[data-section="home"]');
  const appsToggle = root.getElementById("setting-apps-enabled");
  const shareToggle = root.getElementById("setting-share-data");
  const pitchBlackToggle = root.getElementById("setting-pitch-black");
  const blurToggle = root.getElementById("setting-use-blur");
  appsToggle.checked = settings.appsEnabled;
  shareToggle.checked = settings.shareAccountData;
  pitchBlackToggle.checked = settings.pitchBlack;
  blurToggle.checked = settings.useBlur;
  appsButton.hidden = !settings.appsEnabled;
  host.dataset.pitchBlack = String(settings.pitchBlack);
  host.dataset.useBlur = String(settings.useBlur);
  window.__essentialServicesShareAccountDataEnabled = settings.shareAccountData;

  appsToggle.addEventListener("change", () => {
    settings.appsEnabled = appsToggle.checked;
    savePreference("appsEnabled", settings.appsEnabled);
    appsButton.hidden = !settings.appsEnabled;
    if (!settings.appsEnabled && !panels.apps.hidden) homeButton.click();
  });
  shareToggle.addEventListener("change", () => {
    settings.shareAccountData = shareToggle.checked;
    savePreference("shareAccountData", settings.shareAccountData);
    window.__essentialServicesShareAccountDataEnabled = settings.shareAccountData;
  });
  pitchBlackToggle.addEventListener("change", () => {
    settings.pitchBlack = pitchBlackToggle.checked;
    savePreference("pitchBlack", settings.pitchBlack);
    host.dataset.pitchBlack = String(settings.pitchBlack);
  });
  blurToggle.addEventListener("change", () => {
    settings.useBlur = blurToggle.checked;
    savePreference("useBlur", settings.useBlur);
    host.dataset.useBlur = String(settings.useBlur);
  });
  root.getElementById("settings-update").addEventListener("click", () => window.location.reload());
  const categories = {
    music: { titleKey: "music", apps: [] },
    videos: { titleKey: "videos", apps: [] },
    utilities: {
      titleKey: "utilities",
      apps: [
        { name: "Sefirah", description: { es: "De escritorio a móvil", en: "Desktop to mobile" } },
        { name: "Calculator", description: { es: "Calculadora de Fossify", en: "Fossify Calculator" } },
        { name: "Draft", description: { es: "Notas de Sameerasw", en: "Sameerasw's notes" } }
      ]
    },
    games: { titleKey: "games", apps: [] }
  };
  const appList = root.getElementById("app-list");
  const catalogEmpty = root.getElementById("catalog-empty");
  const renderCategory = categoryId => {
    const category = categories[categoryId];
    if (!category) return;
    const categoryTitle = t(category.titleKey);
    root.getElementById("apps-title").textContent = categoryTitle;
    root.getElementById("apps-count").textContent = t("appCount", {
      count: category.apps.length,
      label: t(category.apps.length === 1 ? "appSingular" : "appPlural")
    });
    root.querySelectorAll(".category-tab").forEach(tab => {
      tab.setAttribute("aria-pressed", String(tab.dataset.category === categoryId));
    });
    appList.replaceChildren();
    catalogEmpty.hidden = category.apps.length > 0;
    appList.hidden = category.apps.length === 0;
    if (category.apps.length === 0) {
      root.getElementById("catalog-empty-title").textContent = t("emptyCategory", { category: categoryTitle });
      return;
    }
    for (const app of category.apps) {
      const card = document.createElement("article");
      card.className = "app-card";
      const icon = document.createElement("div");
      icon.className = "app-icon";
      icon.setAttribute("aria-hidden", "true");
      icon.textContent = app.name.slice(0, 1);
      const details = document.createElement("div");
      details.className = "app-details";
      const title = document.createElement("h3");
      title.textContent = app.name;
      const description = document.createElement("p");
      description.textContent = app.description[language];
      details.append(title, description);
      const download = document.createElement("button");
      download.className = "app-download";
      download.type = "button";
      download.disabled = true;
      download.textContent = t("pendingDownload");
      download.title = t("downloadNotConfigured");
      card.append(icon, details, download);
      appList.append(card);
    }
  };
  root.querySelector(".category-tabs").addEventListener("click", event => {
    const tab = event.target.closest("button[data-category]");
    if (tab) renderCategory(tab.dataset.category);
  });
  renderCategory("utilities");
  root.querySelector(".sidebar").addEventListener("click", event => {
    const button = event.target.closest("button[data-section]");
    if (!button) return;
    const section = button.dataset.section;
    for (const [name, panel] of Object.entries(panels)) {
      panel.hidden = section !== name;
    }
    root.querySelectorAll(".sidebar button").forEach(item => {
      if (item === button) item.setAttribute("aria-current", "page");
      else item.removeAttribute("aria-current");
    });
  });
  root.getElementById("minimize").addEventListener("click", () => window.ipc.postMessage("minimize"));
  root.getElementById("maximize").addEventListener("click", () => {
    window.ipc.postMessage("maximize");
    const button = root.getElementById("maximize");
    const maximized = button.dataset.maximized === "true";
    button.dataset.maximized = String(!maximized);
    button.setAttribute("aria-label", maximized ? t("maximize") : t("restore"));
    button.title = maximized ? t("maximize") : t("restore");
    button.innerHTML = maximized
      ? '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="5" y="5" width="14" height="14" rx="1"/></svg>'
      : '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5H5v3M16 5h3v3M19 16v3h-3M8 19H5v-3"/></svg>';
  });
  root.getElementById("close").addEventListener("click", () => window.ipc.postMessage("close"));

  const mount = () => {
    const root = document.documentElement;
    if (root && host.parentElement !== root) root.appendChild(host);
  };
  mount();
  new MutationObserver(mount).observe(document, { childList: true, subtree: true });
})();
"#;

#[derive(Debug)]
enum WindowCommand {
    Minimize,
    ToggleMaximize,
    Drag,
    Close,
}

fn normalize_url(input: &str) -> Result<String, Box<dyn Error>> {
    let input = input.trim();
    if input.is_empty() {
        return Err("La dirección inicial no puede estar vacía.".into());
    }

    let input_lowercase = input.to_ascii_lowercase();
    let candidate =
        if input_lowercase.starts_with("http://") || input_lowercase.starts_with("https://") {
            input.to_owned()
        } else if input.contains("://") {
            return Err("La dirección debe usar HTTP o HTTPS.".into());
        } else {
            format!("https://{input}")
        };

    let url = Url::parse(&candidate)?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("La dirección debe usar HTTP o HTTPS.".into());
    }

    Ok(url.to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    let initial_url = env::args()
        .nth(1)
        .map(|url| normalize_url(&url))
        .transpose()?
        .unwrap_or_else(|| DEFAULT_URL.to_owned());

    let event_loop = EventLoopBuilder::<WindowCommand>::with_user_event().build();
    let event_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("Essential Services Desktop")
        .with_decorations(false)
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)?;

    let initialization_script = WINDOW_BUTTONS_SCRIPT.replace("__TARGET_OS__", TARGET_OS);
    let _webview = WebViewBuilder::new()
        .with_initialization_script_for_main_only(initialization_script, true)
        .with_ipc_handler(move |request: Request<String>| {
            let command = match request.body().as_str() {
                "minimize" => Some(WindowCommand::Minimize),
                "maximize" => Some(WindowCommand::ToggleMaximize),
                "drag" => Some(WindowCommand::Drag),
                "close" => Some(WindowCommand::Close),
                _ => None,
            };
            if let Some(command) = command {
                let _ = event_proxy.send_event(command);
            }
        })
        .with_url(&initial_url)
        .with_new_window_req_handler(|_, _| NewWindowResponse::Allow)
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(WindowCommand::Close) => *control_flow = ControlFlow::Exit,
            Event::UserEvent(WindowCommand::Minimize) => window.set_minimized(true),
            Event::UserEvent(WindowCommand::ToggleMaximize) => {
                window.set_maximized(!window.is_maximized())
            }
            Event::UserEvent(WindowCommand::Drag) => {
                let _ = window.drag_window();
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::normalize_url;

    #[test]
    fn normalizes_a_host_to_https() {
        assert_eq!(
            normalize_url("example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn accepts_uppercase_http_scheme() {
        assert_eq!(
            normalize_url("HTTPS://example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn rejects_non_http_schemes() {
        assert!(normalize_url("ftp://example.com").is_err());
    }
}
