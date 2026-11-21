# Essential Services Desktop

Aplicación de escritorio escrita en Rust que muestra páginas web en una ventana
integrada con WebView2. En Windows, el motor es Microsoft Edge (Chromium), no
Gecko/Firefox.

## Requisitos

- Rust y Cargo (instalables desde [rustup.rs](https://rustup.rs/)).
- Windows 10/11 con el **WebView2 Runtime** instalado.

## Ejecutar

```powershell
cargo run
```

La app abre Google por defecto. Se puede indicar otra página al iniciarla:

```powershell
cargo run -- https://example.com
```

La barra de navegación permite volver, avanzar, recargar, abrir direcciones
HTTP/HTTPS y buscar texto en Google. También acepta `Ctrl+L` para enfocar la
barra y `Ctrl+R` para recargar.

## Crear el ejecutable

```powershell
cargo build --release
```

El ejecutable se genera en `target\release\essential-services-desktop.exe`.
