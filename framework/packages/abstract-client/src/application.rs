
pub struct Application<T: CwEnv, M> {
    account: AbstractAccount<T>,
    module: M,
}

impl<T: CwEnv, M> Deref for Application<T, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<T: CwEnv, M> Application<T, M> {
    pub fn new(account: AbstractAccount<T>, module: M) -> Self {
        Self { account, module }
    }
}

impl<T: CwEnv> Application<T, Manager<T>> {
    pub fn execute(&self, input: &str) -> Result<String, CwError> {
        self.install_module(module_id, init_msg, funds)
    }
}
