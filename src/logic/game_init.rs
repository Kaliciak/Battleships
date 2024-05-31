use futures::{pin_mut, select, Future, FutureExt};

use crate::{
    gui::{Gui, Input},
    net::{connection::Endpoint, result::Result},
};

use super::GameMessage;

async fn await_interruptible<H>(f: impl Future<Output = H>, gui: &mut impl Gui) -> Option<H> {
    let future = f.fuse();
    let esc_future = async {
        loop {
            if let Input::Esc = gui.receive_input().await {
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
    gui: &mut impl Gui,
) -> Option<Endpoint<GameMessage>> {
    if let Some(val) = await_interruptible(f.fuse(), gui).await {
        match val {
            Ok(channel) => Some(channel),
            Err(e) => {
                gui.get_logger().log_message(&e.message);
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
    gui: &mut impl Gui,
) -> Option<Endpoint<GameMessage>> {
    get_channel_from_network_task(
        Endpoint::accept_incoming_connection(addr, passwd, gui.get_logger().as_mut()),
        gui,
    )
    .await
}

pub async fn create_client(
    addr: &str,
    passwd: &str,
    gui: &mut impl Gui,
) -> Option<Endpoint<GameMessage>> {
    get_channel_from_network_task(
        Endpoint::create_connection_to(addr, passwd, gui.get_logger().as_mut()),
        gui,
    )
    .await
}
