use abstract_interface::Abstract;
use cw_orch::{deploy::Deploy, prelude::CwEnv};

use crate::publisher::Publisher;

pub struct AbstractClient<Chain: CwEnv> {
    abstr: Abstract<Chain>,
}

// TODO: Handle errors later.
impl<Chain: CwEnv> AbstractClient<Chain> {
    fn new(chain: Chain) -> Self {
        let abstr = Abstract::load_from(chain).unwrap();
        Self { abstr }
    }

    // TODO: Switch to builder later.
    fn new_publisher(abstr: Abstract<Chain>, namespace: String) -> Publisher<Chain> {
        Publisher::new(namespace)
    }
}
