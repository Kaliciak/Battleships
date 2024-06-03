use async_channel::{unbounded, Receiver, Sender};
use futures::{pin_mut, select, Future, FutureExt};

use super::result::Res;

pub struct AsyncReceiver<T>(pub Receiver<T>);

impl<T> AsyncReceiver<T> {
    pub async fn get(&self) -> Result<T, async_channel::RecvError> {
        self.0.recv().await
    }

    pub async fn consume_in_loop<R: Future<Output = Res<()>>>(
        &mut self,
        f: impl Fn(T) -> R,
    ) -> Res<()> {
        loop {
            f(self.get().await?).await?
        }
    }

    pub async fn with_buffer<R: Future<Output = Res<()>>, Q: Future<Output = Res<()>>, P>(
        &mut self,
        cons: impl Fn(T, Sender<P>) -> R,
        callback: impl FnOnce(AsyncReceiver<P>) -> Q,
    ) -> Res<()> {
        let (sender, receiver) = unbounded();
        let loop_fut = self
            .consume_in_loop(move |m| cons(m, sender.clone()))
            .fuse();
        let callback_fut = callback(AsyncReceiver(receiver)).fuse();

        pin_mut!(loop_fut, callback_fut);

        select! {
            v = loop_fut => v,
            v = callback_fut => v
        }
    }
}
