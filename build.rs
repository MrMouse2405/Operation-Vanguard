use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=VANGUARD_SKIP_UI_BUILD");
    if env::var_os("VANGUARD_SKIP_UI_BUILD").is_some() {
        return;
    }
    for path in [
        "ui/src", "ui/static", "ui/package.json", "ui/bun.lock",
        "ui/vite.config.ts", "ui/tsconfig.json", "ui/components.json",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    let directory = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("ui");
    for arguments in [vec!["install", "--frozen-lockfile"], vec!["run", "build"]] {
        let output = Command::new("bun")
            .args(&arguments)
            .current_dir(&directory)
            .output()
            .expect("Bun is required to build the Vanguard UI. Install it from https://bun.sh");
        assert!(
            output.status.success(),
            "UI command `bun {}` failed:\n{}\n{}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}
