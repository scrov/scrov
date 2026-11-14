//! Desktop shell.
//!
//! The site's axum server runs as a task inside the Tauri process on a port
//! the OS picks, and the webview navigates to it: real server-side rendering
//! in a single native binary, following
//! https://github.com/codeitlikemiley/tauri-leptos-ssr.

use std::path::PathBuf;
use tauri::{Manager, Url};

/// Keeps the embedded server's task so it can be stopped when the window closes.
struct ServerTask(tauri::async_runtime::JoinHandle<()>);

type SetupError = Box<dyn std::error::Error>;

/// Static assets: bundled as a resource in release builds, `../public` in dev.
fn public_dir(app: &tauri::App) -> Result<PathBuf, SetupError> {
    if let Some(dir) = std::env::var_os("SCROV_PUBLIC_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let bundled = app.path().resource_dir()?.join("public");
    if bundled.is_dir() {
        return Ok(bundled);
    }
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../public"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let public = public_dir(app)?;

            let listener = tauri::async_runtime::block_on(async {
                match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                    Ok(listener) => Ok(listener),
                    Err(_) => tokio::net::TcpListener::bind("[::1]:0").await,
                }
            })?;
            let addr = listener.local_addr()?;
            println!("scrov desktop: embedded server on http://{addr}");

            let task = tauri::async_runtime::spawn(async move {
                if let Err(error) = scrov_server::serve(listener, public).await {
                    eprintln!("embedded server stopped: {error}");
                }
            });
            app.manage(ServerTask(task));

            // Wait until the server accepts connections before navigating.
            tauri::async_runtime::block_on(async {
                for _ in 0..50 {
                    if tokio::net::TcpStream::connect(addr).await.is_ok() {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            });

            let window = app
                .get_webview_window("main")
                .ok_or_else(|| SetupError::from("main window not found"))?;
            window.navigate(Url::parse(&format!("http://{addr}/"))?)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(task) = window.try_state::<ServerTask>() {
                    task.0.abort();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running the scrov desktop app");
}
