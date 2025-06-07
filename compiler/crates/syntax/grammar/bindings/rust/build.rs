use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=grammar.js");

    let output = Command::new("npm")
        .arg("run")
        .arg("build")
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "failed to generate grammar: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut c_config = cc::Build::new();
    c_config.std("c11").include("src");
    wasm_flags(&mut c_config);
    c_config.file("src/parser.c");
    c_config.compile("tree-sitter-wipple");
}

// See https://github.com/hydro-project/rust-sitter/tree/main/tool/src/wasm-sysroot
fn wasm_flags(c_config: &mut cc::Build) {
    if env::var("TARGET").unwrap().starts_with("wasm32") {
        c_config
            .flag(format!(
                "--sysroot={}",
                env::current_dir()
                    .unwrap()
                    .join("bindings/rust/wasm-sysroot")
                    .display()
            ))
            .flag("-Wno-everything");
    }
}
