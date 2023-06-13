#[cfg(feature = "node-tests")]
pub use node::*;

#[cfg(feature = "node-tests")]
mod node {
    use std::{env, fs, path::Path, thread::sleep, time::Duration};

    use ctor::{ctor, dtor};

    use duct::cmd;

    // Config
    const JUNO_IMAGE: &str = "ghcr.io/cosmoscontracts/juno:v12.0.0";

    // Defaults for env vars
    const CONTAINER_NAME: &str = "juno_node_1";
    const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

    use uid::Id as IdT;

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct DeployId(());

    #[allow(unused)]
    pub type Id = IdT<DeployId>;

    pub mod state_file {
        use super::{fs, Path};

        pub fn exists(file: &str) -> bool {
            if Path::new(file).exists() {
                log::info!("File found: {}", file);
                true
            } else {
                log::info!("File not found: {}", file);
                false
            }
        }

        pub fn remove(file: &str) {
            if self::exists(file) {
                log::info!("Removing state file: {}", file);
                let _ = fs::remove_file(file);
            }
        }
    }

    pub mod container {
        use super::cmd;

        pub fn find(name: &String) -> bool {
            let read = cmd!("docker", "container", "ls", "--all")
                .pipe(cmd!("grep", name))
                .pipe(cmd!("rev"))
                .pipe(cmd!("cut", "-d", r#" "#, "-f1"))
                .pipe(cmd!("rev"))
                .read();

            match read {
                Ok(val) => {
                    log::info!("Container found: {}", name);
                    val == *name
                }
                Err(_) => false,
            }
        }

        pub fn start(name: &String, image: &String) -> bool {
            if self::find(name) {
                return false;
            }

            // Start Docker with the appropriate ports and the provided environment variables and script
            cmd!(
                "docker",
                "run",
                "-d", // Run the container in detached mode
                "--name",
                name,
                "-p",
                "1317:1317",
                "-p",
                "26656:26656",
                "-p",
                "9090:9090",
                "-e",
                "STAKE_TOKEN=ujunox",
                "-e",
                "UNSAFE_CORS=true",
                image,
                "./setup_and_run.sh",
                "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y"
            )
            .read()
            .is_ok()
        }

        pub fn stop(name: &String) -> bool {
            if !self::find(name) {
                return true;
            }

            log::info!("Stopping container: {}", name);

            let res = cmd!("docker", "container", "stop", name)
                .read()
                .ok()
                .unwrap();

            res == *name
        }

        pub fn remove(name: &String) -> bool {
            if !self::find(name) {
                return true;
            }

            log::info!("Removing container: {}", name);

            let res = cmd!("docker", "container", "rm", name).read().ok().unwrap();

            res == *name
        }

        pub fn ensure_removal(name: &String) {
            if self::stop(name) {
                self::remove(name);
            }
        }
    }

    pub fn docker_container_start() {
        log::info!("Running docker_container_start");

        // Set environment variables
        // this does not seems to be working in this case
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "debug");
        }

        if env::var("CONTAINER_NAME").is_err() {
            env::set_var("CONTAINER_NAME", CONTAINER_NAME);
        }
        let container = env::var("CONTAINER_NAME").unwrap();

        if env::var("JUNO_IMAGE").is_err() {
            env::set_var("JUNO_IMAGE", JUNO_IMAGE);
        }
        let image = env::var("JUNO_IMAGE").unwrap();

        let temp_dir = env::temp_dir();
        let state_file = temp_dir.join("cw_orch_test.json");

        if env::var("STATE_FILE").is_err() {
            env::set_var("STATE_FILE", state_file);
        }

        if env::var("LOCAL_MNEMONIC").is_err() {
            env::set_var("LOCAL_MNEMONIC", LOCAL_MNEMONIC);
        }

        log::info!("Using RUST_LOG: {}", env::var("RUST_LOG").unwrap());
        log::info!("Using CONTAINER_NAME: {}", container);
        log::info!("Using STATE_FILE: {}", env::var("STATE_FILE").unwrap());
        log::info!(
            "Using LOCAL_MNEMONIC: {}",
            env::var("LOCAL_MNEMONIC").unwrap()
        );

        container::start(&container, &image);

        // Wait for docker to start
        sleep(Duration::from_secs(30));
    }

    pub fn docker_container_stop() {
        log::info!("Running docker_container_stop");
        container::ensure_removal(&env::var("CONTAINER_NAME").unwrap());
        let temp_dir = env::temp_dir();
        let expected_state_file = temp_dir.join("cw_orch_test_local.json");
        state_file::remove(expected_state_file.to_str().unwrap());
    }

    #[ctor]
    fn common_start() {
        env_logger::init();
        docker_container_start()
    }

    #[dtor]
    fn common_stop() {
        docker_container_stop()
    }
}
