#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::Duration;
use tauri::{
    Manager,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
};

#[cfg(target_os = "windows")]
const HWND_TOPMOST: HWND = -1isize as HWND;

#[cfg(target_os = "windows")]
fn force_topmost(hwnd_ptr: isize) {
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

#[cfg(target_os = "macos")]
fn raise_above_menu_bar(window: &tauri::WebviewWindow) {
    use objc2::{msg_send, runtime::AnyObject};

    if let Ok(ptr) = window.ns_window() {
        unsafe {
            let ns_window = &*(ptr as *mut AnyObject);
            let _: () = msg_send![ns_window, setLevel: 25_isize];
        }
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
fn lower_webview_memory(window: &tauri::WebviewWindow) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL, ICoreWebView2_19,
    };
    use windows::core::Interface;

    let target = COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL(1);

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
fn low_memory(window: tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    lower_webview_memory(&window);

    #[cfg(not(target_os = "windows"))]
    let _ = window;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            raise_above_menu_bar(&window);
            center_top(&window)?;

            let autostart_item = CheckMenuItem::with_id(
                app,
                "autostart",
                "Start at login",
                true,
                app.autolaunch().is_enabled().unwrap_or(false),
                None::<&str>,
            )?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Cove", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&autostart_item, &separator, &quit])?;

            let autostart_handle = autostart_item.clone();

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Cove")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "autostart" => {
                        let manager = app.autolaunch();
                        let result = if manager.is_enabled().unwrap_or(false) {
                            manager.disable()
                        } else {
                            manager.enable()
                        };
                        if let Err(error) = result {
                            log::warn!("autostart toggle failed: {error}");
                        }
                        let _ = autostart_handle
                            .set_checked(manager.is_enabled().unwrap_or(false));
                    }
                    _ => {}
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

                    thread::spawn(move || {
                        loop {
                            force_topmost(hwnd_ptr);
                            thread::sleep(Duration::from_millis(500));
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![resize_island, low_memory])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}