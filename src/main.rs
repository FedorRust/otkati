use std::path::PathBuf;

fn main() {
    let _ring = otkati::Ring::new(default_otkat_folder());
    // Resident ring process. Capture and a live bind listener are later tickets.
    std::thread::park();
}

fn default_otkat_folder() -> PathBuf {
    let mut folder = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    folder.push("Videos");
    folder.push("Откаты");
    folder
}
