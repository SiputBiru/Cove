use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

#[cfg(target_os = "windows")]
const HWND_TOPMOST: HWND = -1isize as HWND;

fn force_topmost(hwnd_ptr: isize) {
    #[cfg(target_os = "windows")]
    unsafe {
        SetWindowPos(
            hwnd_ptr as _,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

fn center_top(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        let screen_w = monitor.size().width as i32;
        let win_w = window.outer_size()?.width as i32;
        let x = monitor.position().x + (screen_w - win_w) / 2;
        let y = monitor.position().y;
        window.set_position(tauri::PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

#[tauri::command]
fn resize_island(window: tauri::WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    center_top(&window).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = window.hwnd() {
        force_topmost(hwnd.0 as isize);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn set_webview_memory_level(window: &tauri::WebviewWindow, low: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL,
    };
    use windows::core::Interface;

    let target = COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL(if low { 1 } else { 0 });

    let _ = window.with_webview(move |webview| {
        let result = unsafe {
            webview
                .controller()
                .CoreWebView2()
                .and_then(|core| core.cast::<ICoreWebView2_19>())
                .and_then(|memory| memory.SetMemoryUsageTargetLevel(target))
        };

        if let Err(error) = result {
            log::warn!("webview2 memory usage target unavailable: {error}");
        }
    });
}

#[tauri::command]
fn set_idle(window: tauri::WebviewWindow, idle: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    set_webview_memory_level(&window, idle);

    #[cfg(not(target_os = "windows"))]
    let _ = (window, idle);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let window = app.get_webview_window("main").unwrap();
            center_top(&window)?;

            let quit = MenuItem::with_id(app, "quit", "Quit Cove", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Cove")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            #[cfg(target_os = "windows")]
            {
                if let Ok(hwnd) = window.hwnd() {
                    let hwnd_ptr = hwnd.0 as isize;

                    force_topmost(hwnd_ptr);
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::Focused(_) = event {
                            force_topmost(hwnd_ptr);
                        }
                    });

                    thread::spawn(move || loop {
                        force_topmost(hwnd_ptr);
                        thread::sleep(Duration::from_millis(500));
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![resize_island, set_idle])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}