use std::{env, error::Error};

use serde_json::Value;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, WindowBuilder},
};
use url::Url;
use wry::{http::Request, NewWindowResponse, WebViewBuilder};

const DEFAULT_URL: &str = "https://mariohdel14nasa-lang.github.io/essentialservicesweb/";
const APP_CATALOG_URL: &str =
    "https://mariohdel14nasa-lang.github.io/essential-app-catalog/databasedesktop.json";
const PALETTE_URL: &str =
    "https://mariohdel14nasa-lang.github.io/essential-app-catalog/appscolors.json";
const TARGET_OS_KEY: &str = if cfg!(target_os = "macos") {
    "macos"
} else if cfg!(target_os = "windows") {
    "windows"
} else if cfg!(target_os = "linux") {
    "linux"
} else {
    "unknown"
};
const TARGET_OS: &str = if cfg!(target_os = "macos") {
    "macOS"
} else if cfg!(target_os = "windows") {
    "Windows"
} else if cfg!(target_os = "linux") {
    "Linux"
} else {
    "Unknown"
};

#[derive(Debug)]
struct Palette {
    purple: String,
    black: String,
    darkgray: String,
    lightpink: String,
    darkwhite: String,
    purplep: String,
    green: String,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            purple: "#6500f5".to_owned(),
            black: "#000000".to_owned(),
            darkgray: "#201e25".to_owned(),
            lightpink: "#c9adff".to_owned(),
            darkwhite: "#e7e3eb".to_owned(),
            purplep: "#402b7a".to_owned(),
            green: "#448e71".to_owned(),
        }
    }
}

impl Palette {
    fn set(&mut self, name: &str, value: &str) -> bool {
        let Some(color) = parse_color(value) else {
            return false;
        };
        let target = match name.to_ascii_lowercase().as_str() {
            "purple" => &mut self.purple,
            "black" => &mut self.black,
            "darkgray" => &mut self.darkgray,
            "lightpink" => &mut self.lightpink,
            "darkwhite" => &mut self.darkwhite,
            "purplep" => &mut self.purplep,
            "green" => &mut self.green,
            _ => return false,
        };
        *target = color;
        true
    }

    fn css_variables(&self) -> String {
        format!(
            ":host {{ --palette-purple: {}; --palette-black: {}; --palette-darkgray: {}; --palette-lightpink: {}; --palette-darkwhite: {}; --palette-purplep: {}; --palette-green: {}; }}",
            self.purple,
            self.black,
            self.darkgray,
            self.lightpink,
            self.darkwhite,
            self.purplep,
            self.green
        )
    }
}

fn parse_color(value: &str) -> Option<String> {
    let value = value.trim();
    let hex = value.strip_prefix('#')?;
    if !matches!(hex.len(), 6 | 8) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    if hex.len() == 6 {
        return Some(format!("#{hex}"));
    }

    let alpha = u8::from_str_radix(&hex[..2], 16).ok()?;
    let red = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let green = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let blue = u8::from_str_radix(&hex[6..8], 16).ok()?;
    if alpha == u8::MAX {
        Some(format!("#{red:02x}{green:02x}{blue:02x}"))
    } else {
        Some(format!(
            "rgba({red}, {green}, {blue}, {:.3})",
            f32::from(alpha) / f32::from(u8::MAX)
        ))
    }
}

fn collect_json_colors(value: &Value, palette: &mut Palette, count: &mut usize) {
    match value {
        Value::Object(object) => {
            for (name, value) in object {
                if let Some(color) = value.as_str() {
                    if palette.set(name, color) {
                        *count += 1;
                    }
                }
                collect_json_colors(value, palette, count);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_json_colors(value, palette, count);
            }
        }
        _ => {}
    }
}

fn parse_remote_palette(source: &str) -> Result<Palette, Box<dyn Error>> {
    let mut palette = Palette::default();
    let mut count = 0;

    if let Ok(json) = serde_json::from_str::<Value>(source) {
        collect_json_colors(&json, &mut palette, &mut count);
    }

    let mut remaining = source;
    while let Some(start) = remaining.find("color name=\"") {
        remaining = &remaining[start + "color name=\"".len()..];
        let Some(name_end) = remaining.find('"') else {
            break;
        };
        let name = &remaining[..name_end];
        remaining = &remaining[name_end + 1..];
        let Some(value_start) = remaining.find('>') else {
            break;
        };
        remaining = &remaining[value_start + 1..];
        let Some(value_end) = remaining.find("</color>") else {
            break;
        };
        if palette.set(name, &remaining[..value_end]) {
            count += 1;
        }
        remaining = &remaining[value_end + "</color>".len()..];
    }

    if count == 0 {
        return Err("The remote palette contains no recognized colors.".into());
    }
    Ok(palette)
}

fn fetch_palette() -> Result<Palette, Box<dyn Error>> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()?
        .get(PALETTE_URL)
        .send()?
        .error_for_status()?;
    parse_remote_palette(&response.text()?)
}

fn validate_catalog_url(value: &Value, field: &str) -> Result<Option<String>, Box<dyn Error>> {
    let Some(value) = value.as_str() else {
        if value.is_null() {
            return Ok(None);
        }
        return Err(format!("Catalog field '{field}' must be an HTTPS URL.").into());
    };
    let url = Url::parse(value)?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(format!("Catalog field '{field}' must be an HTTPS URL.").into());
    }
    Ok(Some(url.to_string()))
}

fn parse_app_catalog(source: &str, platform: &str) -> Result<Value, Box<dyn Error>> {
    let root: Value = serde_json::from_str(source)?;
    if root.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("Unsupported app catalog schema version.".into());
    }
    let platform_catalog = root
        .get("catalog")
        .and_then(|catalog| catalog.get(platform))
        .and_then(Value::as_object)
        .ok_or("The app catalog does not contain the detected platform.")?;

    let mut result = serde_json::Map::new();
    for category in ["music", "videos", "utilities", "games"] {
        let apps = platform_catalog
            .get(category)
            .and_then(Value::as_array)
            .ok_or_else(|| format!("Catalog category '{category}' must be an array."))?;
        let mut normalized_apps = Vec::with_capacity(apps.len());
        for app in apps {
            let app_object = app
                .as_object()
                .ok_or("Each catalog app must be a JSON object.")?;
            let name = app_object
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .ok_or("Each catalog app must have a non-empty name.")?;
            let description = app_object
                .get("description")
                .and_then(Value::as_str)
                .ok_or("Each catalog app must have a description.")?;
            let icon =
                validate_catalog_url(app_object.get("icon").unwrap_or(&Value::Null), "icon")?;
            let download = validate_catalog_url(
                app_object.get("download").unwrap_or(&Value::Null),
                "download",
            )?;

            let mut normalized_app = serde_json::Map::new();
            normalized_app.insert("name".to_owned(), Value::String(name.to_owned()));
            normalized_app.insert(
                "description".to_owned(),
                Value::String(description.to_owned()),
            );
            if let Some(icon) = icon {
                normalized_app.insert("icon".to_owned(), Value::String(icon));
            }
            if let Some(download) = download {
                normalized_app.insert("download".to_owned(), Value::String(download));
            }
            if let Some(version) = app_object.get("version").and_then(Value::as_str) {
                normalized_app.insert("version".to_owned(), Value::String(version.to_owned()));
            }
            normalized_apps.push(Value::Object(normalized_app));
        }
        result.insert(category.to_owned(), Value::Array(normalized_apps));
    }
    Ok(Value::Object(result))
}

fn fetch_app_catalog(platform: &str) -> Result<Value, Box<dyn Error>> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()?
        .get(APP_CATALOG_URL)
        .send()?
        .error_for_status()?;
    parse_app_catalog(&response.text()?, platform)
}

fn load_app_icon() -> Result<Icon, Box<dyn Error>> {
    let image = image::load_from_memory(include_bytes!("../assets/app-icon.png"))?.into_rgba8();
    let (width, height) = image.dimensions();
    Ok(Icon::from_rgba(image.into_raw(), width, height)?)
}

const WINDOW_BUTTONS_SCRIPT: &str = r#"
(() => {
  if (window.top !== window || window.__essentialServicesControlsInstalled) return;
  window.__essentialServicesControlsInstalled = true;
  const targetOS = "__TARGET_OS__";
  const appCatalog = JSON.parse(__APP_CATALOG_JSON__);

  const host = document.createElement("div");
  host.id = "__essential_services_window_controls";
  host.style.cssText = "all:initial!important;display:block!important;visibility:visible!important;opacity:1!important;position:fixed!important;inset:0!important;z-index:2147483647!important;pointer-events:none!important";

  const root = host.attachShadow({ mode: "closed" });
  root.innerHTML = `
    <style>
      :host { all: initial; }
      __PALETTE_CSS__
      .controls {
        position: fixed; top: 20px; left: 50%; transform: translateX(-50%);
        display: flex; align-items: center; gap: 6px; height: 35px; padding: 0 10px;
        box-sizing: border-box;
        border: 1px solid color-mix(in srgb, var(--palette-darkwhite) 27%, transparent); border-radius: 999px;
        background: var(--palette-darkgray); box-shadow: 0 4px 16px color-mix(in srgb, var(--palette-black) 33%, transparent);
        backdrop-filter: blur(8px); pointer-events: auto; cursor: grab; z-index: 3;
        visibility: visible; opacity: 1;
      }
      .controls button {
        width: 20px; height: 20px; padding: 0; display: grid; place-items: center;
        flex: 0 0 20px;
        border: 1px solid color-mix(in srgb, var(--palette-darkwhite) 27%, transparent); border-radius: 50%;
        background: var(--palette-purplep); color: var(--palette-darkwhite); cursor: pointer;
        transition: background .15s ease, transform .15s ease;
      }
      .controls button:hover { background: var(--palette-purple); transform: scale(1.08); }
      button:focus-visible { outline: 3px solid var(--palette-lightpink); outline-offset: 3px; }
      .controls button.close:hover { background: var(--palette-purplep); }
      svg { fill: none; stroke: currentColor; stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; }
      .controls svg { width: 13px; height: 13px; }
      .sidebar {
        position: fixed; top: 50%; left: 18px; transform: translateY(-50%);
        display: flex; flex-direction: column; align-items: flex-start; gap: 14px;
        pointer-events: auto; z-index: 2;
      }
      .main-nav-group {
        display: flex; flex-direction: column; gap: 5px; padding: 9px;
        border-radius: 28px; background: var(--palette-purple);
        box-shadow: 0 8px 24px color-mix(in srgb, var(--palette-black) 40%, transparent);
      }
      .sidebar button {
        min-width: 132px; height: 50px; padding: 0 14px; display: flex;
        align-items: center; gap: 11px; border: 0; border-radius: 999px;
        background: transparent; color: var(--palette-black); cursor: pointer;
        font: 600 14px "Segoe UI", Arial, sans-serif; text-align: left;
        transition: background .15s ease, color .15s ease, transform .15s ease;
      }
      .main-nav-group button:hover { background: var(--palette-purplep); color: var(--palette-darkwhite); }
      .main-nav-group button[aria-current="page"] { background: var(--palette-darkgray); color: var(--palette-lightpink); }
      .sidebar button[data-section="settings"] {
        background: var(--palette-darkgray); color: var(--palette-lightpink);
        box-shadow: 0 8px 24px color-mix(in srgb, var(--palette-black) 40%, transparent);
      }
      .sidebar button[data-section="settings"]:hover,
      .sidebar button[data-section="settings"][aria-current="page"] {
        background: color-mix(in srgb, var(--palette-darkgray), var(--palette-black) 25%); color: var(--palette-lightpink);
      }
      .sidebar button:focus-visible { outline: 2px solid var(--palette-darkwhite); outline-offset: 2px; }
      .sidebar svg { width: 22px; height: 22px; flex: 0 0 22px; fill: currentColor; stroke: none; }
      .panel {
        position: fixed; left: 184px; top: 50%; transform: translateY(-50%);
        width: 280px; max-height: min(70vh, 500px); overflow: auto;
        padding: 20px; border: 1px solid color-mix(in srgb, var(--palette-darkwhite) 14%, transparent); border-radius: 20px;
        background: var(--palette-darkgray); color: var(--palette-darkwhite); box-shadow: 0 8px 28px color-mix(in srgb, var(--palette-black) 40%, transparent);
        backdrop-filter: blur(12px); pointer-events: auto;
        font: 14px "Segoe UI", Arial, sans-serif;
      }
      .panel[hidden] { display: none; }
      .panel h2 { margin: 0 0 12px; font-size: 18px; }
      .panel p { margin: 0; color: color-mix(in srgb, var(--palette-darkwhite) 78%, var(--palette-darkgray)); line-height: 1.5; }
      .apps-panel {
        inset: 0; width: auto; height: 100vh; max-height: none; padding: 86px 7vw 48px 210px;
        box-sizing: border-box; transform: none; border: 0; border-radius: 0;
        background: var(--palette-darkgray); box-shadow: none; overflow: auto; z-index: 1;
      }
      .catalog-content { width: min(100%, 980px); margin: 0 auto; }
      .catalog-heading { display: flex; align-items: center; gap: 18px; margin-bottom: 52px; }
      .catalog-heading h2 { margin: 0; font-size: clamp(26px, 3vw, 40px); font-weight: 500; }
      .catalog-heading p { margin-top: 5px; font-size: 16px; }
      .catalog-heading-icon {
        width: 76px; height: 76px; display: grid; place-items: center;
        border-radius: 50%; background: var(--palette-black); color: var(--palette-lightpink); flex: 0 0 76px;
      }
      .catalog-heading-icon svg { width: 34px; height: 34px; fill: currentColor; }
      .category-tabs {
        display: flex; flex-wrap: wrap; gap: 8px; margin: 0 0 28px;
        padding: 7px; border-radius: 999px; background: var(--palette-black); width: fit-content;
      }
      .category-tab {
        min-height: 40px; padding: 0 17px; border: 0; border-radius: 999px;
        background: transparent; color: var(--palette-darkwhite); cursor: pointer;
        font: 600 14px "Segoe UI", Arial, sans-serif;
      }
      .category-tab:hover { background: var(--palette-purplep); color: var(--palette-darkwhite); }
      .category-tab[aria-pressed="true"] { background: var(--palette-purple); color: var(--palette-darkwhite); }
      .app-list { display: flex; flex-direction: column; gap: 18px; }
      .app-card {
        display: grid; grid-template-columns: 76px minmax(0, 1fr) 220px;
        align-items: center; gap: 22px; min-height: 112px; padding: 10px 0;
      }
      .app-icon {
        width: 76px; height: 76px; display: grid; place-items: center;
        border-radius: 22px; background: var(--palette-purplep); color: var(--palette-lightpink);
        font: 700 25px "Segoe UI", Arial, sans-serif;
      }
      .app-icon img { width: 100%; height: 100%; border-radius: inherit; object-fit: cover; }
      .app-details h3 { margin: 0 0 4px; font-size: 25px; font-weight: 500; }
      .app-details p { margin: 0; color: color-mix(in srgb, var(--palette-darkwhite) 82%, var(--palette-darkgray)); font-size: 17px; }
      .app-details .app-version { margin-top: 4px; font-size: 13px; }
      .app-download {
        min-height: 72px; display: flex; align-items: center; justify-content: center;
        border: 0; border-radius: 24px; text-decoration: none;
        background: var(--palette-purple); color: var(--palette-black);
        font: 600 20px "Segoe UI", Arial, sans-serif;
      }
      .app-download:hover { filter: brightness(1.12); }
      .app-download.disabled { opacity: .48; cursor: not-allowed; }
      .app-download.disabled:hover { filter: none; }
      .catalog-empty {
        min-height: 180px; display: grid; place-content: center; gap: 10px;
        border: 1px solid color-mix(in srgb, var(--palette-darkwhite) 8%, transparent); border-radius: 20px; background: color-mix(in srgb, var(--palette-darkgray), var(--palette-darkwhite) 5%);
        text-align: center;
      }
      .catalog-empty[hidden], .app-list[hidden] { display: none; }
      .catalog-empty strong { font-size: 17px; font-weight: 600; }
      .catalog-empty p { max-width: 430px; margin: 0 auto; color: color-mix(in srgb, var(--palette-darkwhite) 78%, var(--palette-darkgray)); }
      .settings-panel {
        inset: 0; width: auto; height: 100vh; max-height: none; padding: 48px 7vw;
        box-sizing: border-box; transform: none; border: 0; border-radius: 0;
        background: var(--palette-darkgray); box-shadow: none; overflow: auto; z-index: 1;
      }
      .settings-content { width: min(100%, 980px); margin: 0 auto; }
      .settings-content > h1 { margin: 0 0 30px; font-size: 32px; font-weight: 500; }
      .settings-section { margin: 0 0 26px; }
      .settings-section > h2 {
        margin: 0 0 10px; color: var(--palette-purple); font-size: 14px; font-weight: 600;
      }
      .settings-card {
        overflow: hidden; border-radius: 17px; background: color-mix(in srgb, var(--palette-darkgray), var(--palette-darkwhite) 13%);
      }
      .setting-row {
        min-height: 76px; padding: 14px 18px; display: flex; align-items: center;
        gap: 16px; color: var(--palette-darkwhite); cursor: pointer;
      }
      .setting-row + .setting-row { border-top: 1px solid color-mix(in srgb, var(--palette-darkwhite) 5%, transparent); }
      .setting-icon {
        width: 24px; flex: 0 0 24px; color: var(--palette-purple);
        font-size: 21px; text-align: center;
      }
      .setting-copy { flex: 1; min-width: 0; }
      .setting-copy strong { display: block; margin-bottom: 4px; font-size: 14px; font-weight: 500; }
      .setting-copy small { display: block; color: color-mix(in srgb, var(--palette-darkwhite) 82%, var(--palette-darkgray)); font-size: 12px; line-height: 1.4; }
      .setting-switch {
        appearance: none; width: 48px; height: 28px; margin: 0; flex: 0 0 48px;
        position: relative; border: 0; border-radius: 999px;
        background: color-mix(in srgb, var(--palette-darkgray), var(--palette-darkwhite) 38%); cursor: pointer; transition: background .18s ease;
      }
      .setting-switch::after {
        content: ""; position: absolute; top: 4px; left: 4px;
        width: 20px; height: 20px; border-radius: 50%; background: var(--palette-darkwhite);
        transition: transform .18s ease, background .18s ease;
      }
      .setting-switch:checked { background: var(--palette-purple); }
      .setting-switch:checked::after { transform: translateX(20px); background: var(--palette-black); }
      .setting-switch:disabled { opacity: .48; cursor: not-allowed; }
      .setting-switch:focus-visible { outline: 2px solid var(--palette-lightpink); outline-offset: 3px; }
      .language-select {
        min-width: 150px; padding: 10px 34px 10px 12px; border: 1px solid color-mix(in srgb, var(--palette-darkwhite) 14%, transparent);
        border-radius: 10px; background: var(--palette-darkgray); color: var(--palette-darkwhite);
        font: 14px "Segoe UI", Arial, sans-serif;
      }
      .language-select:focus-visible { outline: 2px solid var(--palette-lightpink); outline-offset: 2px; }
      .settings-update {
        width: 100%; min-height: 64px; padding: 12px 20px; border: 0;
        display: flex; align-items: center; border-radius: 18px;
        background: var(--palette-green); color: var(--palette-darkwhite);
        text-align: left; cursor: pointer; font: 600 16px "Segoe UI", Arial, sans-serif;
        text-decoration: none;
      }
      .settings-update:hover { background: color-mix(in srgb, var(--palette-green), var(--palette-darkwhite) 15%); }
      .credits-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px 28px; }
      .credit strong { display: block; margin-bottom: 3px; color: var(--palette-darkwhite); font-size: 14px; font-weight: 500; }
      .credit span { color: color-mix(in srgb, var(--palette-darkwhite) 78%, var(--palette-darkgray)); font-size: 12px; }
      .about-card { padding: 28px; text-align: center; }
      .about-card h2 { margin: 0 0 12px; font-size: 28px; font-weight: 500; }
      .about-card p { margin: 0 auto 12px; max-width: 620px; color: color-mix(in srgb, var(--palette-darkwhite) 90%, var(--palette-darkgray)); line-height: 1.5; }
      .about-links { display: flex; justify-content: center; flex-wrap: wrap; gap: 10px; margin-top: 20px; }
      .about-links a {
        min-height: 42px; padding: 0 18px; display: inline-flex; align-items: center;
        border-radius: 999px; background: var(--palette-purple); color: var(--palette-black);
        font-weight: 600; text-decoration: none;
      }
      .about-links a:hover { background: color-mix(in srgb, var(--palette-purple), var(--palette-lightpink) 20%); color: var(--palette-darkwhite); }
      :host([data-pitch-black="true"]) .apps-panel,
      :host([data-pitch-black="true"]) .settings-panel { background: var(--palette-black); }
      :host([data-pitch-black="true"]) .settings-card { background: color-mix(in srgb, var(--palette-black), var(--palette-darkwhite) 9%); }
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
        .app-details .app-version { font-size: 12px; }
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
        <a class="settings-update" id="settings-update" href="https://github.com/mariohdel14nasa-lang/essential-app-catalog/releases/latest" target="_blank" rel="noopener noreferrer" data-i18n="update">Actualizar</a>
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
            <h2 id="about-title">Essential Services Beta</h2>
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
      update: "Actualizar", appearance: "Apariencia",
      pitchBlack: "Modo negro puro", pitchBlackDescription: "Usar un fondo negro en las vistas de la aplicación.",
      useBlur: "Usar desenfoque", useBlurDescription: "Aplicar desenfoque progresivo a los elementos de la interfaz.",
      credits: "Créditos", calculatorCreators: "Creadores de Calculator",
      draftCreator: "Creador de Draft y de la app base", kodaCreator: "Creador de Koda",
      sefirahCreator: "Creador de Sefirah", about: "Acerca de",
      aboutDescription: "A service app for other apps",
      aboutCredits: "Desarrollado por mariohdel14nasa-lang con ❤️ desde 🇪🇸",
      website: "Website", viewGithub: "Ver en GitHub",
      emptyCategory: "No hay apps de {category} disponibles todavía",
      catalogLoadError: "No se pudo cargar el catálogo. Comprueba tu conexión e inténtalo al volver a abrir la app.",
      appCount: "{count} {label}", appSingular: "app", appPlural: "apps",
      pendingDownload: "Descarga pendiente", download: "Descargar", downloadNotConfigured: "El enlace de descarga aún no está configurado",
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
      update: "Update", appearance: "Appearance",
      pitchBlack: "Pitch Black Mode", pitchBlackDescription: "Use a black background in the application views.",
      useBlur: "Use Blur", useBlurDescription: "Apply progressive blur to interface elements.",
      credits: "Credits", calculatorCreators: "Creators of Calculator",
      draftCreator: "Creator of Draft and the base app", kodaCreator: "Creator of Koda",
      sefirahCreator: "Creator of Sefirah", about: "About",
      aboutDescription: "A service app for other apps",
      aboutCredits: "Developed by mariohdel14nasa-lang with ❤️ from 🇪🇸",
      website: "Website", viewGithub: "View on GitHub",
      emptyCategory: "No {category} apps available yet",
      catalogLoadError: "The catalog could not be loaded. Check your connection and try again when reopening the app.",
      appCount: "{count} {label}", appSingular: "app", appPlural: "apps",
      pendingDownload: "Download pending", download: "Download", downloadNotConfigured: "The download link has not been configured yet",
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
  const categoryTitles = {
    music: "music",
    videos: "videos",
    utilities: "utilities",
    games: "games"
  };
  const appList = root.getElementById("app-list");
  const catalogEmpty = root.getElementById("catalog-empty");
  const catalogAvailable = appCatalog !== null;
  const renderCategory = categoryId => {
    const titleKey = categoryTitles[categoryId];
    if (!titleKey) return;
    const categoryTitle = t(titleKey);
    const apps = catalogAvailable ? appCatalog[categoryId] : [];
    root.getElementById("apps-title").textContent = categoryTitle;
    root.getElementById("apps-count").textContent = t("appCount", {
      count: apps.length,
      label: t(apps.length === 1 ? "appSingular" : "appPlural")
    });
    root.querySelectorAll(".category-tab").forEach(tab => {
      tab.setAttribute("aria-pressed", String(tab.dataset.category === categoryId));
    });
    appList.replaceChildren();
    catalogEmpty.hidden = apps.length > 0;
    appList.hidden = apps.length === 0;
    if (apps.length === 0) {
      root.getElementById("catalog-empty-title").textContent = catalogAvailable
        ? t("emptyCategory", { category: categoryTitle })
        : t("catalogLoadError");
      return;
    }
    for (const app of apps) {
      const downloadHost = app.download ? new URL(app.download).hostname.toLowerCase() : "";
      const isExampleDownload = downloadHost === "example.com" || downloadHost.endsWith(".example.com");
      const hasDownload = Boolean(app.download) && !isExampleDownload;
      const card = document.createElement("article");
      card.className = "app-card";
      const icon = document.createElement("div");
      icon.className = "app-icon";
      icon.setAttribute("aria-hidden", "true");
      if (app.icon) {
        const image = document.createElement("img");
        image.src = app.icon;
        image.alt = "";
        image.loading = "lazy";
        image.addEventListener("error", () => {
          icon.textContent = app.name.slice(0, 1);
        }, { once: true });
        icon.append(image);
      } else {
        icon.textContent = app.name.slice(0, 1);
      }
      const details = document.createElement("div");
      details.className = "app-details";
      const title = document.createElement("h3");
      title.textContent = app.name;
      const description = document.createElement("p");
      description.textContent = app.description;
      details.append(title, description);
      if (app.version) {
        const version = document.createElement("p");
        version.className = "app-version";
        version.textContent = app.version;
        details.append(version);
      }
      const download = document.createElement(hasDownload ? "a" : "button");
      download.className = `app-download${hasDownload ? "" : " disabled"}`;
      if (hasDownload) {
        download.href = app.download;
        download.target = "_blank";
        download.rel = "noopener noreferrer";
      } else {
        download.type = "button";
        download.disabled = true;
      }
      download.textContent = t(hasDownload ? "download" : "pendingDownload");
      if (!hasDownload) download.title = t("downloadNotConfigured");
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
    let palette = match fetch_palette() {
        Ok(palette) => palette,
        Err(error) => {
            eprintln!("Could not refresh the remote color palette: {error}");
            Palette::default()
        }
    };
    let app_catalog = match fetch_app_catalog(TARGET_OS_KEY) {
        Ok(catalog) => Some(catalog),
        Err(error) => {
            eprintln!("Could not load the remote app catalog: {error}");
            None
        }
    };

    let event_loop = EventLoopBuilder::<WindowCommand>::with_user_event().build();
    let event_proxy = event_loop.create_proxy();
    let app_icon = load_app_icon()?;
    let window = WindowBuilder::new()
        .with_title("Essential Services")
        .with_window_icon(Some(app_icon))
        .with_decorations(false)
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)?;

    let catalog_json = app_catalog
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?
        .unwrap_or_else(|| "null".to_owned());
    let initialization_script = WINDOW_BUTTONS_SCRIPT
        .replace("__TARGET_OS__", TARGET_OS)
        .replace("__PALETTE_CSS__", &palette.css_variables())
        .replace(
            "__APP_CATALOG_JSON__",
            &serde_json::to_string(&catalog_json)?,
        );
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
    use super::{
        normalize_url, parse_app_catalog, parse_color, parse_remote_palette, TARGET_OS_KEY,
    };

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

    #[test]
    fn parses_argb_and_rgb_palette_colors() {
        assert_eq!(parse_color("#FF6200EE").as_deref(), Some("#6200ee"));
        assert_eq!(parse_color("#B8CEDB").as_deref(), Some("#B8CEDB"));
        assert_eq!(parse_color("#402B7A").as_deref(), Some("#402B7A"));
        assert!(parse_color("#12GG00").is_none());
    }

    #[test]
    fn reads_colors_from_the_remote_palette_markup() {
        let response = r##"[
          {
            color name="purple">#FF6200EE</color>
            color name="black">#FF000000</color>
            color name="darkgray">#FF1C1B1F</color>
            color name="lightpink">#FFD0BCFF</color>
            color name="darkwhite">#B8CEDB</color>
            color name="purplep">#402B7A</color>
            color name="green">#488066</color>
          },
        ]"##;
        let palette = parse_remote_palette(response).unwrap();
        assert_eq!(palette.purple, "#6200ee");
        assert_eq!(palette.black, "#000000");
        assert_eq!(palette.darkgray, "#1c1b1f");
        assert_eq!(palette.lightpink, "#d0bcff");
        assert_eq!(palette.darkwhite, "#B8CEDB");
        assert_eq!(palette.purplep, "#402B7A");
        assert_eq!(palette.green, "#488066");
    }

    #[test]
    fn rejects_remote_palette_without_recognized_colors() {
        assert!(parse_remote_palette("{ not a palette }").is_err());
    }

    #[test]
    fn parses_only_the_detected_os_catalog() {
        let response = r#"{
          "schemaVersion": 1,
          "catalog": {
            "windows": {
              "music": [],
              "videos": [],
              "utilities": [{
                "name": "Tool",
                "description": "A desktop tool",
                "icon": "https://example.org/icon.png",
                "download": "https://example.org/tool.zip",
                "version": "2.0"
              }],
              "games": []
            }
          }
        }"#;
        let catalog = parse_app_catalog(response, "windows").unwrap();
        assert_eq!(catalog["utilities"][0]["name"], "Tool");
        assert_eq!(catalog["utilities"][0]["version"], "2.0");
        assert_eq!(catalog["music"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn rejects_insecure_app_download_urls() {
        let response = r#"{
          "schemaVersion": 1,
          "catalog": {
            "windows": {
              "music": [],
              "videos": [],
              "utilities": [{
                "name": "Tool",
                "description": "A desktop tool",
                "download": "http://example.org/tool.zip"
              }],
              "games": []
            }
          }
        }"#;
        assert!(parse_app_catalog(response, "windows").is_err());
    }

    #[test]
    fn rejects_catalogs_with_missing_categories_or_unsupported_schema() {
        assert!(
            parse_app_catalog(r#"{"schemaVersion":2,"catalog":{"windows":{}}}"#, "windows")
                .is_err()
        );
        assert!(parse_app_catalog(
            r#"{"schemaVersion":1,"catalog":{"windows":{"music":[]}}}"#,
            "windows"
        )
        .is_err());
    }

    #[test]
    fn maps_the_build_platform_to_catalog_key() {
        assert!(["windows", "macos", "linux"].contains(&TARGET_OS_KEY));
    }
}
