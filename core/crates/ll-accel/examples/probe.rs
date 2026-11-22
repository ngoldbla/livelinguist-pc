//! Print the measured hardware report as JSON. Used in QA evidence and by
//! `scripts/verify-windows.ps1`; it never includes transcript content.
fn main() {
    let report = ll_accel::detect();
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("serialize hardware report: {error}");
            std::process::exit(1);
        }
    }
}
