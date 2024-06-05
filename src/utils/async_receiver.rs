use async_channel::{unbounded, Receiver, Sender};
use async_std::task::{self, JoinHandle};
use futures::Future;

use super::{result::Res, threads::parallel};

pub struct AsyncReceiver<T: Send + 'static>(pub Receiver<T>);

impl<T: Send + 'static> AsyncReceiver<T> {
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

    pub fn into_bufferred<P, Q, R>(
        mut self,
        cons: R,
    ) -> (AsyncReceiver<P>, JoinHandle<Self>, Reclaim)
    where
        P: Send + 'static,
        Q: Future<Output = Res<()>> + Send + 'static,
        R: Fn(T, Sender<P>) -> Q + Send + Sync + 'static,
    {
        let (sender, receiver) = unbounded();
        let (break_sender, break_receiver) = unbounded();
        let buffer_loop = task::spawn(async move {
            let _ = parallel(
                self.consume_in_loop(move |m| cons(m, sender.clone())),
                async move {
                    let _ = break_receiver.recv().await;
                    println!("Twój stary!");
                    Ok(())
                },
            )
            .await;
            self
        });
        (
            AsyncReceiver(receiver),
            buffer_loop,
            Reclaim {
                sender: break_sender,
            },
        )
    }
}

pub struct Reclaim {
    sender: Sender<()>,
}

impl Reclaim {
    pub async fn ask(&self) {
        let _ = self.sender.send(()).await;
    }
}
