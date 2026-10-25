use std::process::Command;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let runtime_dir_out = Command::new("clang")
            .arg("--print-runtime-dir")
            .output()
            .expect("failed to run clang");

        let runtime_dir = String::from_utf8(runtime_dir_out.stdout).unwrap();

        println!("cargo:rustc-link-search=native={}", runtime_dir.trim());
        println!("cargo:rustc-link-lib=static=clang_rt.osx");
    }
}
