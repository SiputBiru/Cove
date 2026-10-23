use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};
#[cfg(target_os = "windows")]
use tauri::Emitter;
use tauri::{
    Manager,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Shell::SHQueryUserNotificationState;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowLongPtrW,
    GetWindowRect, SM_CXSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowLongPtrW,
    SetWindowPos, WS_EX_NOACTIVATE,
};

// ---------- Shared state ----------

// true while the island is expanded (by frontend)
static OPEN: AtomicBool = AtomicBool::new(false);
// true while a fullscreen app that supports the peek is running
static FS_MODE: AtomicBool = AtomicBool::new(false);
// true while the tiny island is showing above a fullscreen app
#[cfg(target_os = "windows")]
static PEEK: AtomicBool = AtomicBool::new(false);

// ---------- Windows: z-order + fullscreen peek ----------

#[cfg(target_os = "windows")]
const HWND_TOPMOST: HWND = -1isize as HWND;

#[cfg(target_os = "windows")]
const HWND_NOTOPMOST: HWND = -2isize as HWND;

// value by SHQueryUserNotificationState
#[cfg(target_os = "windows")]
const STATE_BUSY: i32 = 2; // fullscreen app, peek is allowed
#[cfg(target_os = "windows")]
const STATE_D3D_FULLSCREEN: i32 = 3; // exclusive fullscreen game, never touch it
#[cfg(target_os = "windows")]
const STATE_PRESENTATION: i32 = 4; // presentation mode, stay hidden
#[cfg(target_os = "windows")]
const STATE_NORMAL: i32 = 5;

// Peek trigger zone (logical px, scaled by DPI at runtime)
#[cfg(target_os = "windows")]
const TRIGGER_HALF_WIDTH: f64 = 110.0; // horizontal hotzone: 110px left/right from center
#[cfg(target_os = "windows")]
const TRIGGER_HEIGHT: i32 = 2; // vertical hotzone: 2px from top edge to trigger
#[cfg(target_os = "windows")]
const KEEP_HALF_WIDTH: f64 = 130.0; // expanded keep-alive zone width while peeking
#[cfg(target_os = "windows")]
const KEEP_HEIGHT: f64 = 28.0; // expanded keep-alive zone height while peeking

// force to stay above all normal apps without stealing focus
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

// drop topmost flag and set Cove right behind the foreground window
#[cfg(target_os = "windows")]
fn force_normal(hwnd_ptr: isize) {
    unsafe {
        SetWindowPos(
            hwnd_ptr as _,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );

        let fg = GetForegroundWindow();
        if !fg.is_null() && fg as isize != hwnd_ptr {
            SetWindowPos(
                hwnd_ptr as _,
                fg,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

// prevent Cove from stealing window focus when clicked during fullscreen modes
#[cfg(target_os = "windows")]
fn set_no_activate(hwnd_ptr: isize, on: bool) {
    unsafe {
        let hwnd = hwnd_ptr as HWND;
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let flag = WS_EX_NOACTIVATE as isize;
        let new_style = if on { style | flag } else { style & !flag };
        if new_style != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
        }
    }
}

// Windows API commands for fullscreen state
#[cfg(target_os = "windows")]
fn notification_state() -> i32 {
    let mut state: i32 = STATE_NORMAL;
    let hr = unsafe { SHQueryUserNotificationState(&mut state) };
    if hr >= 0 { state } else { STATE_NORMAL }
}

// check if cursor currently in the top-center hover activation zones
#[cfg(target_os = "windows")]
fn cursor_zones(scale: f64) -> (bool, bool) {
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) == 0 || pt.y < 0 {
            return (false, false);
        }
        let center_x = GetSystemMetrics(SM_CXSCREEN) / 2;
        let dx = (pt.x - center_x).abs() as f64;
        let in_trigger = pt.y <= TRIGGER_HEIGHT && dx <= TRIGGER_HALF_WIDTH * scale;
        let in_keep = (pt.y as f64) <= KEEP_HEIGHT * scale && dx <= KEEP_HALF_WIDTH * scale;
        (in_trigger, in_keep)
    }
}

// true while any mouse button is physically held down
#[cfg(target_os = "windows")]
fn mouse_down() -> bool {
    unsafe {
        [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON]
            .iter()
            .any(|&vk| (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0)
    }
}

// true when the cursor is outside the island window's rectangle
#[cfg(target_os = "windows")]
fn cursor_outside_window(hwnd_ptr: isize) -> bool {
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetCursorPos(&mut pt) == 0 || GetWindowRect(hwnd_ptr as HWND, &mut rect) == 0 {
            return false;
        }
        pt.x < rect.left || pt.x >= rect.right || pt.y < rect.top || pt.y >= rect.bottom
    }
}

// update local peek state and send frontend event when state changes
#[cfg(target_os = "windows")]
fn set_peek(app: &tauri::AppHandle, peek: bool) {
    if PEEK.swap(peek, Ordering::Relaxed) != peek {
        let _ = app.emit("peek", peek);
    }
}

// reraise window to topmost only if current screen state permits it
#[cfg(target_os = "windows")]
fn raise_if_allowed(hwnd_ptr: isize) {
    let allowed = match notification_state() {
        STATE_BUSY => OPEN.load(Ordering::Relaxed) || PEEK.load(Ordering::Relaxed),
        STATE_D3D_FULLSCREEN | STATE_PRESENTATION => false,
        _ => true,
    };
    if allowed {
        force_topmost(hwnd_ptr);
    }
}

// background loop managing z-order, dock-fighting, mouse-hover peek logic and outside click detection
#[cfg(target_os = "windows")]
fn z_order_loop(app: tauri::AppHandle, hwnd_ptr: isize, scale: f64) {
    const TICK_IDLE: Duration = Duration::from_millis(120);
    const TICK_OPEN: Duration = Duration::from_millis(24); // fast polling so short clicks are caught
    const RECHECK_STATE: Duration = Duration::from_millis(500);
    const RERAISE: Duration = Duration::from_millis(500);
    const HOLD: Duration = Duration::from_millis(300); // hover required before peek triggers
    const LEAVE: Duration = Duration::from_millis(400); // grace period before hiding peek
    const HIDE_DELAY: Duration = Duration::from_millis(500); // allow exit animation to finish before window moves
    const SUSPEND_DELAY: Duration = Duration::from_millis(1000); // let the stow animation finish, then suspend

    let mut state = notification_state();
    let mut applied_state = -1;
    let mut last_state_check = Instant::now();
    let mut last_raise = Instant::now();
    let mut raised = false;
    let mut hover_since: Option<Instant> = None;
    let mut away_since: Option<Instant> = None;
    let mut hide_at: Option<Instant> = None;
    let mut was_down = false;
    let mut quiet_since: Option<Instant> = None;
    let mut suspended = false;

    loop {
        if last_state_check.elapsed() >= RECHECK_STATE {
            state = notification_state();
            last_state_check = Instant::now();
        }

        // react when Windows changes fullscreen
        if state != applied_state {
            applied_state = state;

            let fs = state == STATE_BUSY;
            if fs != FS_MODE.load(Ordering::Relaxed) {
                FS_MODE.store(fs, Ordering::Relaxed);
                let _ = app.emit("fullscreen", fs);
            }

            set_no_activate(hwnd_ptr, fs);
            set_peek(&app, false);
            hover_since = None;
            away_since = None;
            hide_at = None;
            quiet_since = None;

            match state {
                STATE_BUSY => {}
                STATE_D3D_FULLSCREEN | STATE_PRESENTATION => {
                    force_normal(hwnd_ptr);
                    raised = false;
                }
                _ => {
                    force_topmost(hwnd_ptr);
                    raised = true;
                    last_raise = Instant::now();
                }
            }
        }

        // active state
        match state {
            STATE_BUSY => {
                let open = OPEN.load(Ordering::Relaxed);

                // handle hover peek trigger when island is collapsed
                if !open {
                    let (in_trigger, in_keep) = cursor_zones(scale);

                    if PEEK.load(Ordering::Relaxed) {
                        if in_keep {
                            away_since = None;
                        } else if away_since.get_or_insert_with(Instant::now).elapsed() >= LEAVE {
                            set_peek(&app, false);
                            away_since = None;
                        }
                    } else if in_trigger {
                        if hover_since.get_or_insert_with(Instant::now).elapsed() >= HOLD {
                            set_peek(&app, true);
                            hover_since = None;
                        }
                    } else {
                        hover_since = None;
                    }
                }

                let want = open || PEEK.load(Ordering::Relaxed);
                if want {
                    hide_at = None;
                    if !raised || last_raise.elapsed() >= RERAISE {
                        force_topmost(hwnd_ptr);
                        raised = true;
                        last_raise = Instant::now();
                    }
                } else if raised {
                    let deadline = *hide_at.get_or_insert_with(|| Instant::now() + HIDE_DELAY);
                    if Instant::now() >= deadline {
                        force_normal(hwnd_ptr);
                        raised = false;
                        hide_at = None;
                    }
                }
            }
            STATE_D3D_FULLSCREEN | STATE_PRESENTATION => {}
            _ => {
                if last_raise.elapsed() >= RERAISE {
                    force_topmost(hwnd_ptr);
                    last_raise = Instant::now();
                }
            }
        }

        // invisible while stowed in fullscreen, or exclusive fullscreen
        let open_now = OPEN.load(Ordering::Relaxed);
        let peek_now = PEEK.load(Ordering::Relaxed);
        let hidden = match state {
            STATE_D3D_FULLSCREEN | STATE_PRESENTATION => true,
            STATE_BUSY => !open_now && !peek_now && !raised && hover_since.is_none(),
            _ => false,
        };

        if hidden {
            let since = *quiet_since.get_or_insert_with(Instant::now);
            if !suspended && since.elapsed() >= SUSPEND_DELAY {
                set_webview_suspended(&app, true);
                suspended = true;
                log::info!("webview suspended");
            }
        } else {
            quiet_since = None;
            if suspended {
                set_webview_suspended(&app, false);
                suspended = false;
                log::info!("webview resumed");
            }
        }

        // click outside detection, independent of window focus
        let is_open = OPEN.load(Ordering::Relaxed);
        let down = is_open && mouse_down();
        if down && !was_down && cursor_outside_window(hwnd_ptr) {
            let _ = app.emit("outside_click", ());
        }
        was_down = down;

        thread::sleep(if is_open { TICK_OPEN } else { TICK_IDLE });
    }
}

// ---------- macOS ----------

// elevate macOS window level to 25
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

// ---------- Shared helper functions & Tauri Commands ----------

// center window horizontally and set to very top edge of the primary monitor
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

// IPC - resize the island and recenters it at the top
#[tauri::command]
fn resize_island(window: tauri::WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    center_top(&window).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = window.hwnd() {
        raise_if_allowed(hwnd.0 as isize);
    }

    Ok(())
}

// IPC - for frontend when island expands or collapses
#[tauri::command]
fn set_open(open: bool) {
    OPEN.store(open, Ordering::Relaxed);
}

// IPC - check if is in fullscreen
#[tauri::command]
fn fullscreen_mode() -> bool {
    FS_MODE.load(Ordering::Relaxed)
}

#[cfg(target_os = "windows")]
fn configure_webview(window: &tauri::WebviewWindow) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL, ICoreWebView2Profile6, ICoreWebView2Settings3,
        ICoreWebView2Settings5, ICoreWebView2Settings6, ICoreWebView2Settings8, ICoreWebView2_13,
        ICoreWebView2_19,
    };
    use windows::core::Interface;

    let _ = window.with_webview(|webview| {
        let core = match unsafe { webview.controller().CoreWebView2() } {
            Ok(core) => core,
            Err(error) => {
                log::warn!("webview2 core unavailable: {error}");
                return;
            }
        };

        unsafe {
            if let Ok(settings) = core.Settings() {
                // keep devtools usable while developing, closed in release
                let _ = settings.SetAreDevToolsEnabled(cfg!(debug_assertions));
                let _ = settings.SetAreDefaultContextMenusEnabled(false);
                let _ = settings.SetAreDefaultScriptDialogsEnabled(false);
                let _ = settings.SetIsStatusBarEnabled(false);
                let _ = settings.SetIsZoomControlEnabled(false);

                // accelerator keys: Ctrl+P print, Ctrl+F find, F5 reload, ...
                if let Ok(settings3) = settings.cast::<ICoreWebView2Settings3>() {
                    let _ = settings3.SetAreBrowserAcceleratorKeysEnabled(false);
                }
                if let Ok(settings5) = settings.cast::<ICoreWebView2Settings5>() {
                    let _ = settings5.SetIsPinchZoomEnabled(false);
                }
                if let Ok(settings6) = settings.cast::<ICoreWebView2Settings6>() {
                    let _ = settings6.SetIsSwipeNavigationEnabled(false);
                }
                if let Ok(settings8) = settings.cast::<ICoreWebView2Settings8>() {
                    let _ = settings8.SetIsReputationCheckingRequired(false);
                }
            }

            // disable autofill and password saving.
            if let Ok(core13) = core.cast::<ICoreWebView2_13>() {
                if let Ok(profile) = core13.Profile() {
                    if let Ok(profile6) = profile.cast::<ICoreWebView2Profile6>() {
                        let _ = profile6.SetIsGeneralAutofillEnabled(false);
                        let _ = profile6.SetIsPasswordAutosaveEnabled(false);
                    }
                }
            }

            // keep the webview on the low-memory target from the start
            match core.cast::<ICoreWebView2_19>() {
                Ok(memory) => {
                    let target = COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL(1);
                    if let Err(error) = memory.SetMemoryUsageTargetLevel(target) {
                        log::warn!("webview2 memory usage target unavailable: {error}");
                    }
                }
                Err(error) => log::warn!("webview2 memory interface unavailable: {error}"),
            }
        }
    });
}

// suspend rendering while the island is stowed or hidden. resume before a peek is shown.
#[cfg(target_os = "windows")]
fn set_webview_suspended(app: &tauri::AppHandle, suspend: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_3;
    use webview2_com::TrySuspendCompletedHandler;
    use windows::core::Interface;

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let _ = window.with_webview(move |webview| {
        let core = match unsafe {
            webview
                .controller()
                .CoreWebView2()
                .and_then(|core| core.cast::<ICoreWebView2_3>())
        } {
            Ok(core) => core,
            Err(error) => {
                log::warn!("webview2 suspend interface unavailable: {error}");
                return;
            }
        };

        if suspend {
            let handler = TrySuspendCompletedHandler::create(Box::new(|error, succeeded| {
                if error.is_err() || !succeeded {
                    log::warn!("webview2 suspend failed: {error:?}");
                }
                Ok(())
            }));
            if let Err(error) = unsafe { core.TrySuspend(&handler) } {
                log::warn!("webview2 suspend unavailable: {error}");
            }
        } else if let Err(error) = unsafe { core.Resume() } {
            log::warn!("webview2 resume failed: {error}");
        }
    });
}

// ---------- App init ----------

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

            #[cfg(target_os = "windows")]
            configure_webview(&window);

            // Tray menu setup
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
                        let _ = autostart_handle.set_checked(manager.is_enabled().unwrap_or(false));
                    }
                    _ => {}
                })
                .build(app)?;

            // Init windows background z-order supervisor thread
            #[cfg(target_os = "windows")]
            {
                if let Ok(hwnd) = window.hwnd() {
                    let hwnd_ptr = hwnd.0 as isize;
                    let scale = window.scale_factor().unwrap_or(1.0);
                    let app_handle = app.handle().clone();

                    raise_if_allowed(hwnd_ptr);
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::Focused(_) = event {
                            raise_if_allowed(hwnd_ptr);
                        }
                    });

                    thread::spawn(move || z_order_loop(app_handle, hwnd_ptr, scale));
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            resize_island,
            set_open,
            fullscreen_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
