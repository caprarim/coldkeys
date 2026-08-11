use crate::accel;
use crate::store::Bind;
use std::process::Command;

const SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys";
const ENTRY_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
const BASE: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/";

pub fn available() -> bool {
    Command::new("gsettings")
        .args(["get", SCHEMA, "custom-keybindings"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn unquote(raw: &str) -> String {
    let t = raw.trim();
    let t = t.strip_prefix('\'').unwrap_or(t);
    let t = t.strip_suffix('\'').unwrap_or(t);
    t.replace("\\'", "'")
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "\\'"))
}

pub fn list_paths() -> Vec<String> {
    let out = match Command::new("gsettings")
        .args(["get", SCHEMA, "custom-keybindings"])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => return Vec::new(),
    };
    if out.contains("@as") || out == "[]" {
        return Vec::new();
    }
    let inner = out.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(unquote)
        .filter(|s| !s.is_empty())
        .collect()
}

fn set_paths(paths: &[String]) {
    let joined = paths.iter().map(|p| quote(p)).collect::<Vec<_>>().join(", ");
    let value = format!("[{}]", joined);
    let _ = Command::new("gsettings")
        .args(["set", SCHEMA, "custom-keybindings", &value])
        .status();
}

fn get_field(path: &str, field: &str) -> String {
    let target = format!("{}:{}", ENTRY_SCHEMA, path);
    match Command::new("gsettings").args(["get", &target, field]).output() {
        Ok(o) => unquote(&String::from_utf8_lossy(&o.stdout)),
        Err(_) => String::new(),
    }
}

fn set_field(path: &str, field: &str, value: &str) {
    let target = format!("{}:{}", ENTRY_SCHEMA, path);
    let _ = Command::new("gsettings")
        .args(["set", &target, field, &quote(value)])
        .status();
}

fn reset_entry(path: &str) {
    let target = format!("{}:{}", ENTRY_SCHEMA, path);
    for field in ["name", "binding", "command"] {
        let _ = Command::new("gsettings").args(["reset", &target, field]).status();
    }
}

fn key_of(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn import() -> Vec<Bind> {
    let mut binds = Vec::new();
    for path in list_paths() {
        let name = get_field(&path, "name");
        let binding = get_field(&path, "binding");
        let command = get_field(&path, "command");
        if command.is_empty() {
            continue;
        }
        let key = key_of(&path);
        binds.push(Bind {
            id: key.clone(),
            name: if name.is_empty() { key.clone() } else { name },
            command,
            accelerator: accel::from_gnome(&binding),
            enabled: !binding.is_empty(),
            gnome_key: Some(key),
        });
    }
    binds
}

pub fn sync(binds: &[Bind]) {
    let existing = list_paths();
    let mut wanted: Vec<String> = Vec::new();

    for bind in binds {
        if !bind.enabled || bind.accelerator.is_empty() {
            continue;
        }
        let key = bind.gnome_key.clone().unwrap_or_else(|| bind.id.clone());
        let path = format!("{}{}/", BASE, key);
        set_field(&path, "name", &bind.name);
        set_field(&path, "command", &bind.command);
        set_field(&path, "binding", &accel::to_gnome(&bind.accelerator));
        wanted.push(path);
    }

    for path in existing {
        if !wanted.contains(&path) {
            reset_entry(&path);
        }
    }

    set_paths(&wanted);
}
