use async_channel::Sender;

use super::{
    async_receiver::AsyncReceiver, interruptible::{Interruptible, ReturnPoint}, log::{Log, Logger}
};

#[derive(Clone)]
pub struct Context {
    pub exit_point: ReturnPoint<()>,
    pub interrupt_point: ReturnPoint<()>,
    pub logger: Logger,
}

impl Context {
    pub fn new(logger: Logger) -> (Interruptible<()>, Self) {
        let mut interruptible = Interruptible::new();
        let exit_point = interruptible.create_return_point();
        (
            interruptible,
            Context {
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
            Context {
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

    /// Send msg to sender, call interrupt if an error occurred
    pub async fn send_to_or_interrupt<T>(&mut self, msg: T, sender: &mut Sender<T>) {
        if let Err(e) = sender.send(msg).await {
            self.logger.log_message(&format!("Error while sending message: {}", e));
            self.interrupt::<()>().await;
        }
    }

    /// Receive msg from receiver, call interrupt if an error occurred
    pub async fn receive_from_or_interrupt<T>(&mut self, receiver: &mut AsyncReceiver<T>) -> T {
        match receiver.get().await {
            Ok(m) => m,
            Err(e) => {
                self.logger.log_message(&format!("Error while receiving message: {}", e));
                self.interrupt().await
            },
        }
    }
}

