#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use prism_app::AccountService;
use prism_core::{
    AccountSummary, AddAccountInput, Overview, ProviderSummary, PublicError, RefreshStatus,
    UpdateAccountInput,
};
use prism_security::KeyringSecretStore;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, PhysicalSize, State, WindowEvent,
};

struct PanelState {
    pinned: AtomicBool,
}

struct AccountAppState(Mutex<AccountService>);

fn public_error(error: impl std::fmt::Display) -> PublicError {
    PublicError {
        code: "window_error".into(),
        message: error.to_string(),
    }
}

#[tauri::command]
fn get_overview(state: State<'_, AccountAppState>) -> Result<Overview, PublicError> {
    state
        .0
        .lock()
        .map_err(public_error)
        .and_then(|service| service.get_overview())
}

#[tauri::command]
fn get_refresh_status(state: State<'_, AccountAppState>) -> Result<RefreshStatus, PublicError> {
    state
        .0
        .lock()
        .map(|service| service.refresh_status())
        .map_err(public_error)
}

#[tauri::command]
fn refresh(app: tauri::AppHandle) -> Result<(), PublicError> {
    let result = app
        .state::<AccountAppState>()
        .0
        .lock()
        .map_err(public_error)?
        .refresh_demo();
    let status = app
        .state::<AccountAppState>()
        .0
        .lock()
        .map(|service| service.refresh_status())
        .unwrap_or_else(|_| RefreshStatus {
            in_flight: false,
            last_attempt_at: None,
            last_success_at: None,
            retry_after: None,
            error: Some(PublicError {
                code: "service_lock_error".into(),
                message: "Account service became unavailable".into(),
            }),
            stale: false,
            revision: 0,
        });
    app.emit("refresh://state", status.clone())
        .map_err(public_error)?;
    app.emit("demo://refresh", ()).map_err(public_error)?;
    result.map(|_| ())
}

#[tauri::command]
fn list_providers(state: State<'_, AccountAppState>) -> Result<Vec<ProviderSummary>, PublicError> {
    state
        .0
        .lock()
        .map(|service| service.list_providers())
        .map_err(public_error)
}

#[tauri::command]
fn list_accounts(state: State<'_, AccountAppState>) -> Result<Vec<AccountSummary>, PublicError> {
    state
        .0
        .lock()
        .map_err(public_error)
        .and_then(|service| service.list_accounts())
}

#[tauri::command]
fn add_account(
    app: tauri::AppHandle,
    state: State<'_, AccountAppState>,
    input: AddAccountInput,
) -> Result<AccountSummary, PublicError> {
    let result = state
        .0
        .lock()
        .map_err(public_error)
        .and_then(|service| service.add_account(input));
    if result.is_ok() {
        app.emit("accounts://changed", ()).map_err(public_error)?;
    }
    result
}

#[tauri::command]
fn update_account(
    app: tauri::AppHandle,
    state: State<'_, AccountAppState>,
    account_id: String,
    input: UpdateAccountInput,
) -> Result<AccountSummary, PublicError> {
    let result = state
        .0
        .lock()
        .map_err(public_error)
        .and_then(|service| service.update_account(&account_id, input));
    if result.is_ok() {
        app.emit("accounts://changed", ()).map_err(public_error)?;
    }
    result
}

#[tauri::command]
fn remove_account(
    app: tauri::AppHandle,
    state: State<'_, AccountAppState>,
    account_id: String,
) -> Result<bool, PublicError> {
    let result = state
        .0
        .lock()
        .map_err(public_error)
        .and_then(|service| service.remove_account(&account_id));
    if result.is_ok() {
        app.emit("accounts://changed", ()).map_err(public_error)?;
    }
    result
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
            get_refresh_status,
            list_providers,
            list_accounts,
            add_account,
            update_account,
            remove_account,
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
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
            let database = data_dir.join("quotaprism.sqlite");
            let secrets =
                Arc::new(KeyringSecretStore::new("QuotaPrism").map_err(|error| error.to_string())?);
            let service =
                AccountService::open(&database, secrets).map_err(|error| error.to_string())?;
            app.manage(AccountAppState(Mutex::new(service)));
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
