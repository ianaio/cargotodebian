#[cfg_attr(test, mockall::automock)]
pub trait Listener: Send + Sync {
    fn warning(&self, s: String);
    fn info(&self, s: String);
}

pub struct NoOpListener;
impl Listener for NoOpListener {
    fn info(&self, _s: String) {}
    fn warning(&self, _s: String) {}
}

pub struct StdErrListener {
    pub verbose: bool,
}
impl Listener for StdErrListener {
    fn warning(&self, s: String) {
        eprintln!("warning: {}", s);
    }
    fn info(&self, s: String) {
        if self.verbose {
            eprintln!("info: {}", s);
        }
    }
}

