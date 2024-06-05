use std::rc::Rc;

use super::result::Res;

pub trait Log {
    fn log_message(&self, msg: &str) -> Res<()>;
}

#[derive(Clone)]
pub struct Logger {
    pub(crate) log: Rc<dyn Log>,
}

impl Log for Logger {
    fn log_message(&self, msg: &str) -> Res<()> {
        self.log.log_message(msg)
    }
}

impl Logger {
    pub fn new(l: impl Log + 'static) -> Self {
        Self { log: Rc::new(l) }
    }
}

struct PrintLogger {}

impl Log for PrintLogger {
    fn log_message(&self, msg: &str) -> Res<()> {
        println!("{}", msg);
        Ok(())
    }
}

pub fn get_print_logger() -> Logger {
    Logger::new(PrintLogger {})
}
