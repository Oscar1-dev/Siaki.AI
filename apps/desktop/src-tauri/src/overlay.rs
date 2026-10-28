use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, State};

pub const PAUSE_HOTKEY: &str = "Ctrl+Shift+F12";

#[derive(Debug)]
pub struct OverlayState(pub Mutex<OverlayRuntime>);

#[derive(Clone, Copy, Debug)]
pub struct OverlayRuntime {
    pub paused: bool,
    pub active_monitor_index: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    x: i32,
    y: i32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    width: u32,
    height: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDescriptor {
    index: usize,
    name: Option<String>,
    position: Point,
    size: Size,
    scale_factor: f64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayStatus {
    paused: bool,
    active_monitor_index: usize,
    monitors: Vec<MonitorDescriptor>,
    pause_hotkey: &'static str,
}

fn monitors(app: &AppHandle) -> Result<Vec<tauri::Monitor>, String> {
    app.available_monitors()
        .map_err(|error| format!("enumerate monitors: {error}"))
}

fn status(app: &AppHandle, runtime: OverlayRuntime) -> Result<OverlayStatus, String> {
    let monitors = monitors(app)?
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| MonitorDescriptor {
            index,
            name: monitor.name().map(ToOwned::to_owned),
            position: Point {
                x: monitor.position().x,
                y: monitor.position().y,
            },
            size: Size {
                width: monitor.size().width,
                height: monitor.size().height,
            },
            scale_factor: monitor.scale_factor(),
        })
        .collect();
    Ok(OverlayStatus {
        paused: runtime.paused,
        active_monitor_index: runtime.active_monitor_index,
        monitors,
        pause_hotkey: PAUSE_HOTKEY,
    })
}

pub fn move_to_monitor(app: &AppHandle, index: usize) -> Result<(), String> {
    let monitor = monitors(app)?
        .into_iter()
        .nth(index)
        .ok_or_else(|| "monitor index is unavailable".to_owned())?;
    let overlay = app
        .get_webview_window("overlay")
        .ok_or_else(|| "overlay window unavailable".to_owned())?;
    overlay
        .set_position(PhysicalPosition::new(
            monitor.position().x,
            monitor.position().y,
        ))
        .map_err(|e| e.to_string())?;
    overlay
        .set_size(PhysicalSize::new(
            monitor.size().width,
            monitor.size().height,
        ))
        .map_err(|e| e.to_string())?;
    let controller = app
        .get_webview_window("main")
        .ok_or_else(|| "controller window unavailable".to_owned())?;
    controller
        .set_position(PhysicalPosition::new(
            monitor.position().x + 24,
            monitor.position().y + 24,
        ))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_paused(app: &AppHandle, paused: bool) -> Result<(), String> {
    for label in ["overlay", "main"] {
        let window = app
            .get_webview_window(label)
            .ok_or_else(|| format!("{label} window unavailable"))?;
        if paused { window.hide() } else { window.show() }.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_overlay_status(
    app: AppHandle,
    state: State<'_, OverlayState>,
) -> Result<OverlayStatus, String> {
    let runtime = *state
        .0
        .lock()
        .map_err(|_| "overlay state unavailable".to_owned())?;
    status(&app, runtime)
}

#[tauri::command]
pub fn move_overlay_to_monitor(
    app: AppHandle,
    state: State<'_, OverlayState>,
    monitor_index: usize,
) -> Result<OverlayStatus, String> {
    move_to_monitor(&app, monitor_index)?;
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "overlay state unavailable".to_owned())?;
    runtime.active_monitor_index = monitor_index;
    log::info!(target: "saiki.overlay", "event=overlay.monitor_changed index={monitor_index}");
    status(&app, *runtime)
}

#[tauri::command]
pub fn set_overlay_paused(
    app: AppHandle,
    state: State<'_, OverlayState>,
    paused: bool,
) -> Result<OverlayStatus, String> {
    apply_paused(&app, paused)?;
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "overlay state unavailable".to_owned())?;
    runtime.paused = paused;
    log::info!(target: "saiki.overlay", "event=overlay.pause_changed paused={paused}");
    status(&app, *runtime)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hotkey_is_explicit() {
        assert_eq!(PAUSE_HOTKEY, "Ctrl+Shift+F12");
    }
}
