use tauri::Manager;

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
  center_top(&window).map_err(|e| e.to_string())
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
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![resize_island])
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
