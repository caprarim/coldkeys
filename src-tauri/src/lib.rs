mod accel;
mod store;

#[cfg(target_os = "linux")]
mod gnome;

use std::process::Command;
use std::sync::Mutex;
use store::Bind;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};

struct AppState {
    binds: Mutex<Vec<Bind>>,
    dir: Mutex<std::path::PathBuf>,
}

fn spawn_command(command: &str) -> Result<(), String> {
    if command.trim().is_empty() {
        return Err("This shortcut has no command".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", command])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .arg("-c")
            .arg(format!("setsid {} >/dev/null 2>&1 &", command))
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

fn persist(state: &AppState) -> Result<(), String> {
    let binds = state.binds.lock().unwrap();
    let dir = state.dir.lock().unwrap();
    store::save(&dir, &binds)?;
    #[cfg(target_os = "linux")]
    if gnome::available() {
        gnome::sync(&binds);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_shortcuts(app: &AppHandle, binds: &[Bind]) {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    for bind in binds {
        if !bind.enabled || bind.accelerator.is_empty() {
            continue;
        }
        let shortcut = match Shortcut::from_str(&bind.accelerator) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let command = bind.command.clone();
        let _ = manager.on_shortcut(shortcut, move |_app, _sc, event| {
            if event.state == ShortcutState::Pressed {
                let _ = spawn_command(&command);
            }
        });
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_shortcuts(_app: &AppHandle, _binds: &[Bind]) {}

#[tauri::command]
fn platform() -> String {
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
}

#[tauri::command]
fn list_binds(state: State<AppState>) -> Vec<Bind> {
    state.binds.lock().unwrap().clone()
}

#[tauri::command]
fn upsert_bind(app: AppHandle, state: State<AppState>, bind: Bind) -> Result<Vec<Bind>, String> {
    {
        let mut binds = state.binds.lock().unwrap();
        let clash = binds.iter().any(|b| {
            b.id != bind.id
                && b.enabled
                && !bind.accelerator.is_empty()
                && b.accelerator.eq_ignore_ascii_case(&bind.accelerator)
        });
        if clash {
            return Err(format!("{} is already used by another shortcut", bind.accelerator));
        }
        match binds.iter_mut().find(|b| b.id == bind.id) {
            Some(existing) => {
                existing.name = bind.name;
                existing.command = bind.command;
                existing.accelerator = bind.accelerator;
                existing.enabled = bind.enabled;
            }
            None => binds.push(bind),
        }
    }
    persist(&state)?;
    let binds = state.binds.lock().unwrap().clone();
    apply_shortcuts(&app, &binds);
    Ok(binds)
}

#[tauri::command]
fn delete_bind(app: AppHandle, state: State<AppState>, id: String) -> Result<Vec<Bind>, String> {
    {
        let mut binds = state.binds.lock().unwrap();
        binds.retain(|b| b.id != id);
    }
    persist(&state)?;
    let binds = state.binds.lock().unwrap().clone();
    apply_shortcuts(&app, &binds);
    Ok(binds)
}

#[tauri::command]
fn import_system(app: AppHandle, state: State<AppState>) -> Result<Vec<Bind>, String> {
    #[cfg(target_os = "linux")]
    {
        if !gnome::available() {
            return Err("GNOME settings are not available on this system".to_string());
        }
        let found = gnome::import();
        let mut binds = state.binds.lock().unwrap();
        for item in found {
            if !binds.iter().any(|b| b.id == item.id) {
                binds.push(item);
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        return Err("Importing system shortcuts only works on Linux".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        {
            let binds = state.binds.lock().unwrap();
            let dir = state.dir.lock().unwrap();
            store::save(&dir, &binds)?;
        }
        let binds = state.binds.lock().unwrap().clone();
        apply_shortcuts(&app, &binds);
        Ok(binds)
    }
}

#[tauri::command]
fn run_bind(state: State<AppState>, id: String) -> Result<(), String> {
    let binds = state.binds.lock().unwrap();
    let bind = binds.iter().find(|b| b.id == id).ok_or("Shortcut not found")?;
    spawn_command(&bind.command)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let fresh = !store::config_path(&dir).exists();
            let mut binds = store::load(&dir);

            #[cfg(target_os = "linux")]
            if fresh && gnome::available() {
                binds = gnome::import();
                let _ = store::save(&dir, &binds);
            }
            #[cfg(not(target_os = "linux"))]
            let _ = fresh;

            apply_shortcuts(app.handle(), &binds);

            app.manage(AppState {
                binds: Mutex::new(binds),
                dir: Mutex::new(dir),
            });

            let open = MenuItem::with_id(app, "open", "Open ColdKeys", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("ColdKeys")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform,
            list_binds,
            upsert_bind,
            delete_bind,
            import_system,
            run_bind
        ])
        .run(tauri::generate_context!())
        .expect("error while running ColdKeys");
}
