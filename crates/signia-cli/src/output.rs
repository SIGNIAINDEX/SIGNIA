use std::io::{self, Write};

use serde::Serialize;
use termcolor::{ColorChoice, StandardStream};

static mut JSON_MODE: bool = false;

pub fn init(json: bool) {
    unsafe { JSON_MODE = json; }
}

pub fn is_json() -> bool {
    unsafe { JSON_MODE }
}

pub fn print<T: Serialize>(value: &T) -> anyhow::Result<()> {
    if is_json() {
        let s = serde_json::to_string_pretty(value)?;
        println!("{s}");
        return Ok(());
    }
    let s = serde_json::to_string_pretty(value)?;
    println!("{s}");
    Ok(())
}

pub fn eprintln_line(msg: &str) {
    let _ = writeln!(io::stderr(), "{msg}");
}

pub fn stdout() -> StandardStream {
    StandardStream::stdout(ColorChoice::Auto)
}
