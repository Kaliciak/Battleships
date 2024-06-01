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
