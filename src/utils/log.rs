use std::{cell::RefCell, rc::Rc};

pub trait Log {
    fn log_message(&mut self, msg: &str);
}

#[derive(Clone)]
pub struct Logger {
    pub(crate) log: Rc<RefCell<dyn Log>>,
}

impl Log for Logger {
    fn log_message(&mut self, msg: &str) {
        self.log.borrow_mut().log_message(msg)
    }
}

impl Logger {
    pub fn new(l: impl Log + 'static) -> Self {
        Self {
            log: Rc::new(RefCell::new(l)),
        }
    }
}

struct PrintLogger {}

impl Log for PrintLogger {
    fn log_message(&mut self, msg: &str) {
        println!("{}", msg);
    }
}

pub fn get_print_logger() -> Logger {
    Logger::new(PrintLogger {})
}
