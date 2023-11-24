const APP_ID: &str = "tester:app";
const APP_VERSION: &str = "1.0.0";
abstract_app::gen_app_better_mock!(MockApp, APP_ID, APP_VERSION, &[]);

fn main() {}
