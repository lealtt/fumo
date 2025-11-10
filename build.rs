use std::{env, fs, process::Command};

fn extract_dep_version(lock_contents: &str, dep_name: &str) -> Option<String> {
    let mut is_target = false;
    for line in lock_contents.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            is_target = false;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name = \"") {
            let name = rest.trim_end_matches('"');
            is_target = name == dep_name;
            continue;
        }
        if is_target {
            if let Some(rest) = trimmed.strip_prefix("version = \"") {
                return Some(rest.trim_end_matches('"').to_string());
            }
        }
    }
    None
}

fn main() {
    let rustc_path = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(&rustc_path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=FUMO_RUSTC_VERSION={version}");

    let lock_contents = fs::read_to_string("Cargo.lock").unwrap_or_default();
    for dep in ["serenity", "poise"] {
        if let Some(ver) = extract_dep_version(&lock_contents, dep) {
            let var_name = format!("FUMO_{}_VERSION", dep.to_ascii_uppercase());
            println!("cargo:rustc-env={var_name}={ver}");
        }
    }
}
