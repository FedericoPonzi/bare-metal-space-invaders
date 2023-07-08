use std::env;

fn main() {
    const LINKER_FILE: &str = "LINKER_FILE";
    const DEFAULT_PATH: &str = "../linker/rpi_3b+.ld";
    let linker_file = env::var(LINKER_FILE).unwrap_or(DEFAULT_PATH.to_string());
    println!("cargo:rerun-if-changed={}", linker_file);
}
