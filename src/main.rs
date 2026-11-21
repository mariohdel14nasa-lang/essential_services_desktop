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

const WINDOW_BUTTONS_SCRIPT: &str = r#"
(() => {
  if (window.top !== window || window.__essentialServicesControlsInstalled) return;
  window.__essentialServicesControlsInstalled = true;

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
        backdrop-filter: blur(8px); pointer-events: auto; cursor: grab;
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
        pointer-events: auto;
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
      .apps-panel { width: 420px; }
      .catalog-heading { display: flex; align-items: center; gap: 10px; margin-bottom: 18px; }
      .catalog-heading h2 { margin: 0; }
      .catalog-heading p { margin-top: 3px; font-size: 13px; }
      .catalog-heading-icon {
        width: 42px; height: 42px; display: grid; place-items: center;
        border-radius: 50%; background: #17121e; color: #c9adff;
      }
      .catalog-heading-icon svg { width: 22px; height: 22px; fill: currentColor; }
      .os-folders { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
      .os-folder {
        min-height: 92px; padding: 12px 8px; display: flex; flex-direction: column;
        align-items: center; justify-content: center; gap: 8px;
        border: 1px solid #ffffff20; border-radius: 16px;
        background: #ffffff08; color: #e7e3eb; cursor: pointer;
        font: 600 12px "Segoe UI", Arial, sans-serif;
      }
      .os-folder:hover, .os-folder:focus-visible { background: #6500f566; border-color: #b98aff; }
      .os-folder svg { width: 25px; height: 25px; fill: #b98aff; }
      .apps-folder-view[hidden] { display: none; }
      .catalog-back {
        margin: 0 0 14px; padding: 6px 10px; border: 0; border-radius: 9px;
        background: #ffffff12; color: #e7e3eb; cursor: pointer;
        font: 600 13px "Segoe UI", Arial, sans-serif;
      }
      .catalog-back:hover { background: #ffffff24; }
      @media (max-width: 520px) {
        .sidebar { left: 8px; }
        .main-nav-group { padding: 7px; }
        .sidebar button { min-width: 48px; width: 48px; justify-content: center; padding: 0; }
        .sidebar button span { display: none; }
        .panel, .apps-panel { left: 72px; width: min(420px, calc(100vw - 84px)); }
        .os-folders { grid-template-columns: 1fr; }
        .os-folder { min-height: 54px; flex-direction: row; justify-content: flex-start; padding: 10px 14px; }
      }
    </style>
    <div class="controls" aria-label="Controles de ventana">
      <button id="minimize" aria-label="Minimizar" title="Minimizar">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14"/></svg>
      </button>
      <button id="maximize" aria-label="Maximizar" title="Maximizar">
        <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="5" y="5" width="14" height="14" rx="1"/></svg>
      </button>
      <button class="close" id="close" aria-label="Cerrar" title="Cerrar">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18"/></svg>
      </button>
    </div>
    <nav class="sidebar" aria-label="Menú principal">
      <div class="main-nav-group">
        <button data-section="home" aria-current="page">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5.85,17.1q1.27-.98 2.85-1.54T12,15t3.3,.56t2.85,1.54q.88-1.03 1.36-2.33T20,12q0-3.33-2.34-5.66T12,4Q8.68,4 6.34,6.34T4,12q0,1.47 .49,2.78q.49,1.3 1.36,2.33Zm3.66-5.11Q8.5,10.98 8.5,9.5q0-1.48 1.01-2.49T12,6t2.49,1.01T15.5,9.5t-1.01,2.49T12,13q-1.47,0-2.49-1.01ZM12,22q-2.07,0-3.9-.79q-1.82-.79-3.17-2.14Q3.58,17.73 2.79,15.9T2,12Q2,9.92 2.79,8.1Q3.58,6.27 4.93,4.93Q6.28,3.57 8.1,2.79Q9.93,2 12,2t3.9,.79t3.17,2.14q1.35,1.35 2.14,3.17Q22,9.92 22,12t-.79,3.9q-.79,1.82-2.14,3.17q-1.35,1.35-3.17,2.14T12,22Zm2.5-2.39q1.18-.39 2.15-1.11q-.98-.73-2.15-1.11T12,17t-2.5,.39Q8.33,17.77 7.35,18.5q.98,.73 2.15,1.11T12,20t2.5-.39Zm-1.42-9.04Q13.5,10.15 13.5,9.5T13.08,8.42T12,8t-1.07,.42T10.5,9.5t.43,1.07T12,11t1.08-.43ZM12,9.5Zm0,9Z"/></svg>
          <span>Home</span>
        </button>
        <button data-section="apps">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4.59,19.41Q4,18.83 4,18t.59-1.41T6,16q.83,0 1.41,.59Q8,17.18 8,18T7.41,19.41T6,20t-1.41-.59Zm6,0Q10,18.83 10,18t.59-1.41T12,16t1.41,.59Q14,17.18 14,18t-.59,1.41T12,20t-1.41-.59Zm6,0Q16,18.83 16,18t.59-1.41T18,16t1.41,.59Q20,17.18 20,18t-.59,1.41T18,20t-1.41-.59Zm-12-6Q4,12.83 4,12t.59-1.41T6,10q.83,0 1.41,.59Q8,11.18 8,12t-.59,1.41T6,14t-1.41-.59Zm6,0Q10,12.83 10,12t.59-1.41T12,10t1.41,.59Q14,11.18 14,12t-.59,1.41T12,14t-1.41-.59Zm6,0Q16,12.83 16,12t.59-1.41T18,10t1.41,.59Q20,11.18 20,12t-.59,1.41T18,14t-1.41-.59Zm-12-6Q4,6.82 4,6T4.59,4.59T6,4q.83,0 1.41,.59Q8,5.18 8,6t-.59,1.41T6,8t-1.41-.59Zm6,0Q10,6.82 10,6t.59-1.41T12,4t1.41,.59Q14,5.18 14,6t-.59,1.41T12,8t-1.41-.59Zm6,0Q16,6.82 16,6t.59-1.41T18,4t1.41,.59Q20,5.18 20,6t-.59,1.41T18,8t-1.41-.59Z"/></svg>
          <span>Apps</span>
        </button>
      </div>
      <button data-section="settings">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10.83,22q-0.68,0-1.16-0.45t-0.59-1.1L8.85,18.8q-0.32-0.13-0.61-0.3T7.68,18.13L6.13,18.77Q5.5,19.05 4.88,18.83T3.9,18.02L2.73,15.98q-0.35-0.57-0.2-1.22T3.2,13.68l1.33-1Q4.5,12.5 4.5,12.34q0-0.16 0-0.34t0-0.34t0.03-0.34l-1.33-1Q2.68,9.9 2.53,9.25t0.2-1.23L3.9,5.97Q4.25,5.4 4.88,5.18T6.13,5.22L7.68,5.88Q7.95,5.68 8.25,5.5t0.6-0.3L9.08,3.55q0.1-0.65 0.59-1.1T10.83,2h2.35q0.68 0 1.16,0.45t0.59,1.1L15.15,5.2q0.33,0.13 0.61,0.3t0.56,0.38l1.55-0.65q0.63-0.27 1.25-0.05t0.98,0.8l1.18,2.05q0.35,0.58 0.2,1.23T20.8,10.33l-1.32,1q0.02,0.18 0.02,0.34t0,0.34t0,0.34t-0.05,0.34l1.32,1q0.52,0.43 0.68,1.08t-0.2,1.22l-1.2,2.05q-0.35,0.58-0.98,0.8t-1.25-0.05l-1.5-0.65q-0.27,0.2-0.57,0.38t-0.6,0.3l-0.22,1.65q-0.1,0.65-0.59,1.1T13.18,22H10.83ZM11,20h1.98l0.35-2.65q0.78-0.2 1.44-0.59t1.21-0.94l2.47,1.03l0.98-1.7l-2.15-1.63q0.13-0.35 0.17-0.74T17.5,12t-0.05-0.79q-0.05-0.39-0.17-0.74l2.15-1.63l-0.98-1.7L15.98,8.2q-0.55-0.57-1.21-0.96T13.33,6.65L13,4H11.03L10.68,6.65Q9.9,6.85 9.24,7.24T8.03,8.17L5.55,7.15L4.58,8.85l2.15,1.6Q6.6,10.83 6.55,11.2T6.5,12q0,0.4 0.05,0.77t0.17,0.75L4.58,15.15l0.98,1.7L8.03,15.8q0.55,0.58 1.21,0.96t1.44,0.59L11,20Zm1.05-4.5q1.45,0 2.47-1.03T15.55,12T14.53,9.52T12.05,8.5q-1.47,0-2.49,1.02T8.55,12t1.01,2.47t2.49,1.03ZM12,12Z"/></svg>
        <span>Ajustes</span>
      </button>
    </nav>
    <section class="panel apps-panel" id="apps-panel" aria-labelledby="apps-title" hidden>
      <div id="apps-catalog">
        <div class="catalog-heading">
          <div class="catalog-heading-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24"><path d="M10 4H3a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h18a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2H12l-2-2z"/></svg>
          </div>
          <div><h2 id="apps-title">Catálogo de Apps</h2><p>Selecciona una plataforma</p></div>
        </div>
        <div class="os-folders" aria-label="Carpetas por sistema operativo">
          <button class="os-folder" data-os="macOS">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10.2 4.1c.7-.9 1.8-1.6 2.8-1.7.1 1.2-.3 2.4-1 3.2-.7.8-1.8 1.5-2.9 1.4-.1-1.1.4-2.2 1.1-2.9ZM19.7 16.8c-.5 1.1-.8 1.6-1.4 2.6-.9 1.4-2.1 3.1-3.7 3.1-1.4 0-1.8-.9-3.7-.9s-2.3.9-3.7.9c-1.6 0-2.8-1.5-3.7-2.9-2.6-3.9-2.9-8.5-1.3-11 1.1-1.8 2.9-2.8 4.6-2.8 1.5 0 2.4.9 4.1.9 1.6 0 2.6-.9 4.4-.9 1.5 0 3.1.8 4.2 2.2-3.7 2-3.1 7.3.2 8.8Z"/></svg>
            <span>macOS</span>
          </button>
          <button class="os-folder" data-os="Windows">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M2 5.2 10.8 4v7.5H2V5.2Zm10.2-1.4L22 2.4v9.1h-9.8V3.8ZM2 12.8h8.8v7.5L2 19.1v-6.3Zm10.2 0H22v9.1l-9.8-1.4v-7.7Z"/></svg>
            <span>Windows</span>
          </button>
          <button class="os-folder" data-os="Linux">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 2a5 5 0 0 0-5 5v4.1c-.5.8-1.2 1.4-2 2.1-1.1 1-2 2-2 3.8 0 2.3 1.7 4 4 4 1 0 2-.4 3-1h4c1 .6 2 1 3 1 2.3 0 4-1.7 4-4 0-1.8-.9-2.8-2-3.8-.8-.7-1.5-1.3-2-2.1V7a5 5 0 0 0-5-5Zm-2 7a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm4 0a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm-5 4h6a3 3 0 0 1-6 0Z"/></svg>
            <span>Linux</span>
          </button>
        </div>
      </div>
      <div class="apps-folder-view" id="apps-folder-view" hidden>
        <button class="catalog-back" id="apps-back" type="button">← Volver al catálogo</button>
        <h2 id="apps-folder-title"></h2>
        <p>No hay aplicaciones añadidas para este sistema operativo todavía.</p>
      </div>
    </section>
    <section class="panel" id="settings-panel" aria-labelledby="settings-title" hidden>
      <h2 id="settings-title">Ajustes</h2>
      <p>Los ajustes de la aplicación estarán disponibles aquí.</p>
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
  const appsCatalog = root.getElementById("apps-catalog");
  const appsFolderView = root.getElementById("apps-folder-view");
  root.querySelector(".os-folders").addEventListener("click", event => {
    const folder = event.target.closest("button[data-os]");
    if (!folder) return;
    root.getElementById("apps-folder-title").textContent = folder.dataset.os;
    appsCatalog.hidden = true;
    appsFolderView.hidden = false;
  });
  root.getElementById("apps-back").addEventListener("click", () => {
    appsFolderView.hidden = true;
    appsCatalog.hidden = false;
  });
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
    button.setAttribute("aria-label", maximized ? "Maximizar" : "Restaurar");
    button.title = maximized ? "Maximizar" : "Restaurar";
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

    let _webview = WebViewBuilder::new()
        .with_initialization_script_for_main_only(WINDOW_BUTTONS_SCRIPT, true)
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
