pub fn main() {
    // run `cargo test` in this directory; this is used so that `cargo test` in the top-level directory actually runs the tests here
    std::process::Command::new("cargo")
        .arg("test")
        .current_dir("tests")
        .status()
        .expect("Failed to run tests in the subdirectory");
}
