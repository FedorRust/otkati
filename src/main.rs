fn main() {
    let _ring = otkati::Ring::new(otkati::Ring::default_otkat_folder());
    // Resident ring process. Capture and a live bind listener are later tickets.
    std::thread::park();
}
