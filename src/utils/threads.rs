use std::thread;

use futures::{select, FutureExt};

use futures::{
    future::{select, Either},
    pin_mut, Future,
};

use super::result::Res;

pub async fn spawn_thread_async<F, T>(f: F) -> Res<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = async_channel::unbounded::<T>();

    let _ = thread::spawn(move || {
        let _ = sender.send_blocking(f());
    });

    Ok(receiver.recv().await?)
}

pub async fn select_first<K, L>(
    f1: impl Future<Output = Res<K>>,
    f2: impl Future<Output = Res<L>>,
) -> Res<Either<K, L>> {
    pin_mut!(f1, f2);

    match select(f1, f2).await {
        Either::Left(a) => Ok(Either::Left(a.0?)),
        Either::Right(a) => Ok(Either::Right(a.0?)),
    }
}

pub async fn merge<K, L>(
    f1: impl Future<Output = Res<K>>,
    f2: impl Future<Output = Res<L>>,
) -> Res<(K, L)> {
    let f1_fuse = f1.fuse();
    let f2_fuse = f2.fuse();

    pin_mut!(f1_fuse, f2_fuse);
    let mut results: (Option<K>, Option<L>) = (None, None);

    loop {
        select! {
            k = f1_fuse => {
                results.0 = Some(k?);
            }
            l = f2_fuse => {
                results.1 = Some(l?)
            }
            complete => {
                return Ok((results.0.unwrap(), results.1.unwrap()));
            }
        }
    }
}
