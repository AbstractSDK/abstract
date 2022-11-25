// Example custom build script.
fn main() {
    const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
    if CONTRACT_VERSION != "0.2.0-beta.5" {
        panic!("remove migration state-changes for next release")
    }
}
