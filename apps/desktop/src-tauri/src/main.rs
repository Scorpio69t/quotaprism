#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use prism_core::{Overview, PublicError, QuotaProvider};
use prism_providers::DemoProvider;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, PhysicalSize, State, WindowEvent,
};

struct PanelState {
    pinned: AtomicBool,
}

struct DemoState(Mutex<Overview>);

fn public_error(error: impl std::fmt::Display) -> PublicError {
    PublicError {
        code: "window_error".into(),
        message: error.to_string(),
    }
}

#[tauri::command]
fn get_overview(state: State<'_, DemoState>) -> Result<Overview, PublicError> {
    state
        .0
        .lock()
        .map(|overview| overview.clone())
        .map_err(public_error)
}

#[tauri::command]
fn refresh(app: tauri::AppHandle) -> Result<(), PublicError> {
    let updated = DemoProvider.fetch_overview()?;
    *app.state::<DemoState>().0.lock().map_err(public_error)? = updated;
    app.emit("demo://refresh", ()).map_err(public_error)
}

#[tauri::command]
fn set_panel_pinned(state: State<'_, PanelState>, pinned: bool) {
    state.pinned.store(pinned, Ordering::SeqCst);
}

#[tauri::command]
fn hide_panel(app: tauri::AppHandle) -> Result<(), PublicError> {
    window(&app, "panel")?.hide().map_err(public_error)
}

#[tauri::command]
fn show_dashboard(app: tauri::AppHandle) -> Result<(), PublicError> {
    window(&app, "panel")?.hide().map_err(public_error)?;
    let main = window(&app, "main")?;
    main.show().map_err(public_error)?;
    main.unminimize().map_err(public_error)?;
    main.set_focus().map_err(public_error)
}

fn window(app: &tauri::AppHandle, label: &str) -> Result<tauri::WebviewWindow, PublicError> {
    app.get_webview_window(label)
        .ok_or_else(|| public_error(format!("Window '{label}' is unavailable")))
}

fn toggle_panel(
    app: &tauri::AppHandle,
    position: PhysicalPosition<f64>,
) -> Result<(), PublicError> {
    let panel = window(app, "panel")?;
    if panel.is_visible().map_err(public_error)? {
        return panel.hide().map_err(public_error);
    }
    let monitors = panel.available_monitors().map_err(public_error)?;
    let monitor = monitors
        .iter()
        .find(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            position.x >= f64::from(origin.x)
                && position.y >= f64::from(origin.y)
                && position.x < f64::from(origin.x) + f64::from(size.width)
                && position.y < f64::from(origin.y) + f64::from(size.height)
        })
        .ok_or_else(|| public_error("No monitor found for the tray"))?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let width = (400.0 * scale).round().min(f64::from(area.size.width)) as u32;
    let height = (580.0 * scale).round().min(f64::from(area.size.height)) as u32;
    let left = area.position.x;
    let top = area.position.y;
    let right = left + area.size.width as i32;
    let bottom = top + area.size.height as i32;
    let (x, y) = panel_position(
        (left, top, right, bottom),
        (width as i32, height as i32),
        (position.x as i32, position.y as i32),
    );
    panel
        .set_size(PhysicalSize::new(width, height))
        .map_err(public_error)?;
    panel
        .set_position(PhysicalPosition::new(x, y))
        .map_err(public_error)?;
    panel.show().map_err(public_error)?;
    panel.set_focus().map_err(public_error)
}

fn panel_position(
    (left, top, right, bottom): (i32, i32, i32, i32),
    (width, height): (i32, i32),
    (x, y): (i32, i32),
) -> (i32, i32) {
    let centered_x = (x - width / 2).clamp(left, right - width);
    let centered_y = (y - height / 2).clamp(top, bottom - height);
    let distances = [
        (x - left).abs(),
        (x - right).abs(),
        (y - top).abs(),
        (y - bottom).abs(),
    ];
    let nearest = distances
        .iter()
        .enumerate()
        .min_by_key(|(_, distance)| *distance)
        .map(|(index, _)| index);
    match nearest {
        Some(0) => (left, centered_y),
        Some(1) => (right - width, centered_y),
        Some(2) => (centered_x, top),
        _ => (centered_x, bottom - height),
    }
}

fn log_result(result: Result<(), PublicError>) {
    if let Err(error) = result {
        eprintln!("{error}");
    }
}

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            log_result(show_dashboard(app.clone()));
        }))
        .manage(PanelState {
            pinned: AtomicBool::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            get_overview,
            refresh,
            set_panel_pinned,
            hide_panel,
            show_dashboard
        ])
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                if let Err(error) = window.hide() {
                    eprintln!("Failed to hide window: {error}");
                }
            }
            WindowEvent::Focused(false)
                if window.label() == "panel"
                    && !window.state::<PanelState>().pinned.load(Ordering::SeqCst) =>
            {
                if let Err(error) = window.hide() {
                    eprintln!("Failed to hide panel: {error}");
                }
            }
            _ => {}
        })
        .setup(|app| {
            app.manage(DemoState(Mutex::new(DemoProvider.fetch_overview()?)));
            let open = MenuItem::with_id(app, "open", "Open dashboard", true, None::<&str>)?;
            let refresh_item =
                MenuItem::with_id(app, "refresh", "Refresh demo", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &refresh_item, &quit])?;
            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or("Missing application icon")?;
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("QuotaPrism · Demo")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => log_result(show_dashboard(app.clone())),
                    "refresh" => log_result(refresh(app.clone())),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        position,
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        log_result(toggle_panel(tray.app_handle(), position));
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("Failed to run QuotaPrism: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::panel_position;

    #[test]
    fn panel_anchors_to_each_taskbar_edge_and_stays_in_work_area() {
        let area = (0, 0, 1920, 1040);
        assert_eq!(panel_position(area, (400, 580), (1800, 1060)), (1520, 460));
        assert_eq!(panel_position(area, (400, 580), (960, -20)), (760, 0));
        assert_eq!(panel_position(area, (400, 580), (-20, 500)), (0, 210));
        assert_eq!(panel_position(area, (400, 580), (1940, 500)), (1520, 210));
    }
}
