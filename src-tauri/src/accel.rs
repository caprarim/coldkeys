const SPECIAL: &[(&str, &str)] = &[
    ("Enter", "Return"),
    ("Space", "space"),
    ("Quote", "apostrophe"),
    ("Comma", "comma"),
    ("Period", "period"),
    ("Slash", "slash"),
    ("Semicolon", "semicolon"),
    ("Backslash", "backslash"),
    ("BracketLeft", "bracketleft"),
    ("BracketRight", "bracketright"),
    ("Minus", "minus"),
    ("Equal", "equal"),
    ("Backquote", "grave"),
    ("Tab", "Tab"),
    ("Escape", "Escape"),
    ("Backspace", "BackSpace"),
    ("Delete", "Delete"),
    ("Insert", "Insert"),
    ("Home", "Home"),
    ("End", "End"),
    ("PageUp", "Page_Up"),
    ("PageDown", "Page_Down"),
    ("ArrowUp", "Up"),
    ("ArrowDown", "Down"),
    ("ArrowLeft", "Left"),
    ("ArrowRight", "Right"),
];

fn split(accel: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = accel.split('+').map(|p| p.trim()).collect();
    let mut mods = Vec::new();
    let mut key = String::new();
    for part in parts {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods.push("Control".to_string()),
            "shift" => mods.push("Shift".to_string()),
            "alt" | "option" => mods.push("Alt".to_string()),
            "super" | "meta" | "cmd" | "command" => mods.push("Super".to_string()),
            _ => key = part.to_string(),
        }
    }
    (mods, key)
}

pub fn to_gnome(accel: &str) -> String {
    let (mods, key) = split(accel);
    let mut out = String::new();
    for m in &mods {
        out.push_str(&format!("<{}>", m));
    }
    let gkey = SPECIAL
        .iter()
        .find(|(t, _)| t.eq_ignore_ascii_case(&key))
        .map(|(_, g)| g.to_string())
        .unwrap_or_else(|| {
            if key.len() == 1 && key.chars().next().unwrap().is_ascii_alphabetic() {
                key.to_ascii_lowercase()
            } else if key.len() > 3 && key.to_ascii_lowercase().starts_with("key") {
                key[3..].to_ascii_lowercase()
            } else if key.len() > 5 && key.to_ascii_lowercase().starts_with("digit") {
                key[5..].to_string()
            } else {
                key.clone()
            }
        });
    out.push_str(&gkey);
    out
}

pub fn from_gnome(binding: &str) -> String {
    let mut mods = Vec::new();
    let mut rest = binding;
    loop {
        let lower = rest.to_ascii_lowercase();
        if lower.starts_with("<control>") || lower.starts_with("<ctrl>") || lower.starts_with("<primary>") {
            mods.push("Control");
            rest = &rest[rest.find('>').unwrap() + 1..];
        } else if lower.starts_with("<shift>") {
            mods.push("Shift");
            rest = &rest[rest.find('>').unwrap() + 1..];
        } else if lower.starts_with("<alt>") {
            mods.push("Alt");
            rest = &rest[rest.find('>').unwrap() + 1..];
        } else if lower.starts_with("<super>") || lower.starts_with("<mod4>") {
            mods.push("Super");
            rest = &rest[rest.find('>').unwrap() + 1..];
        } else {
            break;
        }
    }

    let key = SPECIAL
        .iter()
        .find(|(_, g)| g.eq_ignore_ascii_case(rest))
        .map(|(t, _)| t.to_string())
        .unwrap_or_else(|| {
            if rest.len() == 1 && rest.chars().next().unwrap().is_ascii_alphabetic() {
                rest.to_ascii_uppercase()
            } else {
                rest.to_string()
            }
        });

    let mut out: Vec<String> = mods.iter().map(|m| m.to_string()).collect();
    out.push(key);
    out.join("+")
}
