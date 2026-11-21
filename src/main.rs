use std::{env, error::Error};

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use url::Url;
use wry::{NewWindowResponse, WebViewBuilder};

const DEFAULT_URL: &str = "https://www.google.com/";

const BROWSER_CHROME_SCRIPT: &str = r#"
(() => {
  if (window.top !== window || document.getElementById("__essential_services_chrome")) return;

  const host = document.createElement("div");
  host.id = "__essential_services_chrome";
  host.style.cssText = "all:initial!important;position:fixed!important;top:0!important;left:0!important;right:0!important;z-index:2147483647!important;display:block!important";

  const root = host.attachShadow({ mode: "closed" });
  root.innerHTML = `
    <style>
      :host { all: initial; }
      * { box-sizing: border-box; }
      .bar {
        height: 54px; width: 100%; padding: 0 14px; display: flex;
        align-items: center; gap: 8px; background: #101827;
        border-bottom: 1px solid #283449; color: #e7edf7;
        font: 13px "Segoe UI", Arial, sans-serif;
        box-shadow: 0 2px 8px #0003;
      }
      .brand { display: flex; align-items: center; gap: 8px; margin-right: 8px; white-space: nowrap; font-weight: 600; }
      .mark { width: 9px; height: 9px; border-radius: 50%; background: #6ea8fe; }
      button {
        height: 34px; min-width: 36px; border: 0; border-radius: 7px;
        background: transparent; color: #d7dfeb; cursor: pointer;
        font: inherit; padding: 0 10px;
      }
      button:hover { background: #263248; }
      .address {
        display: flex; align-items: center; flex: 1; min-width: 80px;
        height: 36px; padding: 0 12px; border: 1px solid #344258;
        border-radius: 9px; background: #1b2638;
      }
      .address:focus-within { border-color: #75aaff; }
      input {
        width: 100%; border: 0; outline: 0; background: transparent;
        color: #f3f6fb; font: inherit;
      }
      input::selection { background: #486da5; }
      .go { background: #376fc1; color: white; }
      .go:hover { background: #4781d5; }
      @media (max-width: 600px) {
        .brand { display: none; }
        .bar { padding: 0 8px; gap: 4px; }
      }
    </style>
    <header class="bar" aria-label="Navegación">
      <div class="brand"><span class="mark"></span><span>Essential Services</span></div>
      <button id="back" title="Atrás" aria-label="Atrás">Atrás</button>
      <button id="forward" title="Adelante" aria-label="Adelante">Adelante</button>
      <button id="reload" title="Recargar" aria-label="Recargar">Recargar</button>
      <form class="address" id="form">
        <input id="url" aria-label="Dirección o búsqueda" spellcheck="false"
          autocomplete="off" placeholder="Escribe una dirección o busca en la web">
      </form>
      <button class="go" id="go" aria-label="Ir">Ir</button>
    </header>`;

  const input = root.getElementById("url");
  const syncAddress = () => { input.value = window.location.href; };
  const navigate = () => {
    const text = input.value.trim();
    if (!text) return;

    let destination;
    const looksLikeHost =
      /^(localhost|(?:[\w-]+\.)+[a-z]{2,}|\d{1,3}(?:\.\d{1,3}){3}|\[[\da-f:]+\])(?::\d+)?(?:[/?#]|$)/i.test(text);
    if (/^https?:\/\//i.test(text)) {
      destination = text;
    } else if (looksLikeHost) {
      destination = `https://${text}`;
    } else if (/^[a-z][a-z\d+.-]*:/i.test(text)) {
      input.setCustomValidity("Solo se permiten direcciones HTTP y HTTPS.");
      input.reportValidity();
      return;
    } else {
      destination = `https://www.google.com/search?q=${encodeURIComponent(text)}`;
    }

    try {
      const parsed = new URL(destination);
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return;
      window.location.assign(parsed.href);
    } catch {
      input.setCustomValidity("Escribe una dirección válida o una búsqueda.");
      input.reportValidity();
    }
  };

  input.addEventListener("input", () => input.setCustomValidity(""));
  root.getElementById("form").addEventListener("submit", event => {
    event.preventDefault();
    navigate();
  });
  root.getElementById("go").addEventListener("click", navigate);
  root.getElementById("back").addEventListener("click", () => history.back());
  root.getElementById("forward").addEventListener("click", () => history.forward());
  root.getElementById("reload").addEventListener("click", () => location.reload());
  window.addEventListener("popstate", syncAddress);
  window.addEventListener("hashchange", syncAddress);

  for (const method of ["pushState", "replaceState"]) {
    const original = history[method];
    history[method] = function (...args) {
      const result = original.apply(this, args);
      syncAddress();
      return result;
    };
  }

  window.addEventListener("keydown", event => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "l") {
      event.preventDefault();
      input.focus();
      input.select();
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r") {
      event.preventDefault();
      location.reload();
    }
  }, true);

  const mount = () => {
    if (!host.isConnected && document.documentElement) {
      document.documentElement.appendChild(host);
      syncAddress();
    }
  };
  mount();
  new MutationObserver(mount).observe(document, { childList: true, subtree: true });
})();
"#;

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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Essential Services Desktop")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new()
        .with_initialization_script_for_main_only(BROWSER_CHROME_SCRIPT, true)
        .with_url(&initial_url)
        .with_new_window_req_handler(|_, _| NewWindowResponse::Allow)
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
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
