use futures::{pin_mut, select, Future, FutureExt};

use crate::{
    gui::{get_gui_input, Gui},
    net::{connection::Endpoint, result::Result},
    utils::{input::InputFilter, log::Log},
};

use super::GameMessage;

async fn await_interruptible<H>(
    f: impl Future<Output = H>,
    filter: &mut InputFilter,
    gui: &mut impl Gui,
) -> H {
    let future = f.fuse();
    let esc_future = async {
        loop {
            get_gui_input(filter, gui).await;
        }
    }
    .fuse();

    pin_mut!(future, esc_future);
    select! {
        val = esc_future => val,
        val = future => val
    }
}

async fn get_channel_from_network_task(
    f: impl Future<Output = Result<Endpoint<GameMessage>>>,
    filter: &mut InputFilter,
    gui: &mut impl Gui,
) -> Option<Endpoint<GameMessage>> {
    match await_interruptible(f.fuse(), filter, gui).await {
        Ok(channel) => Some(channel),
        Err(e) => {
            gui.get_logger().log_message(&e.message);
            None
        }
    }
}

pub async fn create_host(
    addr: &str,
    passwd: &str,
    filter: &mut InputFilter,
    gui: &mut impl Gui,
) -> Option<Endpoint<GameMessage>> {
    get_channel_from_network_task(
        Endpoint::accept_incoming_connection(addr, passwd, filter.clone()),
        filter,
        gui,
    )
    .await
}

pub async fn create_client(
    addr: &str,
    passwd: &str,
    filter: &mut InputFilter,
    gui: &mut impl Gui,
) -> Option<Endpoint<GameMessage>> {
    get_channel_from_network_task(
        Endpoint::create_connection_to(addr, passwd, filter.clone()),
        filter,
        gui,
    )
    .await
}
