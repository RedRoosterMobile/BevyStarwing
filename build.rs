// Recordings are optional. Dropping in a WAV and rebuilding embeds it for native + web.
use std::{env, fs, path::PathBuf};

fn main() {
    let directory =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/voice/panther");
    println!("cargo:rerun-if-changed={}", directory.display());
    let names = [
        "launch",
        "kill",
        "roll",
        "hit",
        "low_shields",
        "victory",
        "defeat",
    ];
    let mut table = String::from("[");
    for name in names {
        let path = directory.join(format!("{name}.wav"));
        if path.is_file() {
            table.push_str(&format!("Some(&include_bytes!({:?})[..]),", path));
        } else {
            table.push_str("None,");
        }
    }
    table.push(']');
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("panther_voices.rs"),
        table,
    )
    .unwrap();
}
