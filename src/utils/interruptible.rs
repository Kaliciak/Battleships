use async_channel::{Receiver, Sender};
use async_std::task;
use futures::{
    future::{select, Either},
    pin_mut, select, Future, FutureExt,
};

struct InfTask {}

impl Future for InfTask {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        task::Poll::Pending
    }
}

async fn stop<T>() -> T {
    InfTask {}.await;
    unreachable!();
}

pub struct Interruptible<R> {
    sender: Sender<R>,
    receiver: Receiver<R>,
}

#[derive(Clone)]
pub struct ReturnPoint<R> {
    sender: Sender<R>,
}

impl<R> Interruptible<R> {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Interruptible { sender, receiver }
    }
    pub async fn run(self, task: impl Future<Output = R>) -> R {
        let recfut = self.receiver.recv().fuse();
        let taskfut = task.fuse();

        pin_mut!(recfut, taskfut);

        select! {
            val = taskfut => val,
            val = recfut => val.unwrap()
        }
    }

    pub fn create_return_point(&mut self) -> ReturnPoint<R> {
        ReturnPoint {
            sender: self.sender.clone(),
        }
    }
}

impl<R> ReturnPoint<R> {
    pub async fn jump<T>(&mut self, ret_val: R) -> T {
        self.sender.send(ret_val).await.expect("Channel is closed");
        stop().await
    }
}
