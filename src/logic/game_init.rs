use futures::{pin_mut, select, Future, FutureExt};

use crate::{
    ui::{UI, Input},
    net::{connection::Endpoint, result::Result},
};

use super::GameMessage;

async fn await_interruptible<H>(f: impl Future<Output = H>, ui: &mut impl UI) -> Option<H> {
    let future = f.fuse();
    let esc_future = async {
        loop {
            if let Input::Esc = ui.receive_input().await {
                return;
            }
        }
    }
    .fuse();

    pin_mut!(future, esc_future);
    select! {
        () = esc_future => None,
        val = future => Some(val)
    }
}

async fn get_channel_from_network_task(
    f: impl Future<Output = Result<Endpoint<GameMessage>>>,
    ui: &mut impl UI,
) -> Option<Endpoint<GameMessage>> {
    if let Some(val) = await_interruptible(f.fuse(), ui).await {
        match val {
            Ok(channel) => Some(channel),
            Err(e) => {
                ui.get_logger().log_message(&e.message);
                None
            }
        }
    } else {
        None
    }
}

pub async fn create_host(
    addr: &str,
    passwd: &str,
    ui: &mut impl UI,
) -> Option<Endpoint<GameMessage>> {
    get_channel_from_network_task(
        Endpoint::accept_incoming_connection(addr, passwd, ui.get_logger().as_mut()),
        ui,
    )
    .await
}

pub async fn create_client(
    addr: &str,
    passwd: &str,
    ui: &mut impl UI,
) -> Option<Endpoint<GameMessage>> {
    get_channel_from_network_task(
        Endpoint::create_connection_to(addr, passwd, ui.get_logger().as_mut()),
        ui,
    )
    .await
}
