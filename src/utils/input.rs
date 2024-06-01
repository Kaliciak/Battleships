use super::{
    interruptible::{Interruptible, ReturnPoint},
    log::Logger,
};

#[derive(Clone)]
pub struct InputFilter {
    exit_point: ReturnPoint<()>,
    interrupt_point: ReturnPoint<()>,
    pub logger: Logger,
}

impl InputFilter {
    pub fn new(logger: Logger) -> (Interruptible<()>, Self) {
        let mut interruptible = Interruptible::new();
        let exit_point = interruptible.create_return_point();
        (
            interruptible,
            InputFilter {
                interrupt_point: exit_point.clone(),
                exit_point,
                logger,
            },
        )
    }

    pub fn advance(&mut self) -> (Interruptible<()>, Self) {
        let mut interruptible = Interruptible::new();
        let interruption_point = interruptible.create_return_point();
        (
            interruptible,
            InputFilter {
                interrupt_point: interruption_point.clone(),
                exit_point: self.exit_point.clone(),
                logger: self.logger.clone(),
            },
        )
    }

    pub async fn interrupt<T>(&mut self) -> T {
        self.interrupt_point.jump(()).await
    }

    pub async fn exit<T>(&mut self) -> T {
        self.exit_point.jump(()).await
    }
}
