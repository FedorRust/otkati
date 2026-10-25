use std::path::PathBuf;
use std::process::Command;

fn tool_args(flag: &str) -> Vec<String> {
    let output = Command::new("pkg-config")
        .args([flag, "Qt6Qml", "Qt6Core"])
        .output()
        .expect("pkg-config Qt6Qml Qt6Core");
    assert!(output.status.success(), "pkg-config {flag} failed");
    String::from_utf8(output.stdout)
        .expect("pkg-config output")
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let header = manifest.join("face/files/files.h");
    let source = manifest.join("face/files/files.cpp");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let plugin_dir = manifest.join("face/Otkati/Files");
    std::fs::create_dir_all(&plugin_dir).unwrap();

    let moc_cpp = out_dir.join("moc_files.cpp");
    let mut moc = Command::new("/usr/lib/qt6/moc");
    moc.args(tool_args("--cflags"))
        .arg(&header)
        .arg("-o")
        .arg(&moc_cpp);
    let moc_status = moc.status().expect("qt6 moc");
    assert!(moc_status.success(), "moc failed");

    let plugin = plugin_dir.join("libotkatifiles.so");
    let mut compile = Command::new("g++");
    compile
        .args(["-shared", "-fPIC"])
        .args(tool_args("--cflags"))
        .arg("-I")
        .arg(source.parent().unwrap())
        .arg("-I")
        .arg(&out_dir)
        .arg(&source)
        .arg("-o")
        .arg(&plugin)
        .args(tool_args("--libs"));
    let compile_status = compile.status().expect("g++");
    assert!(
        compile_status.success(),
        "g++ failed to build the face file plugin"
    );

    println!("cargo:rerun-if-changed=face/files/files.h");
    println!("cargo:rerun-if-changed=face/files/files.cpp");
}
