use std::thread;

use futures::{
    future::{select, Either},
    pin_mut, Future,
};

use super::{log::Log, result::Res};

pub async fn spawn_thread_async<F, T, R>(f: F, logger: R) -> Res<T>
where
    R: Log + Send + 'static,
    F: FnOnce(R) -> T + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = async_channel::unbounded::<T>();

    let _ = thread::spawn(move || {
        sender.send_blocking(f(logger)).unwrap();
    });

    Ok(receiver.recv().await?)
}

pub async fn parallel<K, L>(
    f1: impl Future<Output = Res<K>>,
    f2: impl Future<Output = Res<L>>,
) -> Res<Either<K, L>> {
    pin_mut!(f1, f2);

    match select(f1, f2).await {
        Either::Left(a) => Ok(Either::Left(a.0?)),
        Either::Right(a) => Ok(Either::Right(a.0?)),
    }
}
