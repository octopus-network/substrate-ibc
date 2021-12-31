use super::*;
use ibc::{
	application::ics20_fungible_token_transfer::context::Ics20Context,
	ics26_routing::context::Ics26Context,
};

#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub tmp: u8,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		Self { _pd: PhantomData::default(), tmp: 0 }
	}
}

impl<T: Config> Ics26Context for Context<T> {}

impl<T: Config> Ics20Context for Context<T> {}

pub trait ModuleCallbacks {
	// fn on_chan_open_try(
	// 	index: usize,
	// 	order: ChannelOrder,
	// 	connection_hops: Vec<H256>,
	// 	port_identifier: Vec<u8>,
	// 	channel_identifier: H256,
	// 	counterparty_port_identifier: Vec<u8>,
	// 	counterparty_channel_identifier: H256,
	// 	version: Vec<u8>,
	// 	counterparty_version: Vec<u8>,
	// );
	// fn on_chan_open_ack(
	// 	index: usize,
	// 	port_identifier: Vec<u8>,
	// 	channel_identifier: H256,
	// 	version: Vec<u8>,
	// );
	// fn on_chan_open_confirm(index: usize, port_identifier: Vec<u8>, channel_identifier: H256);
	// fn on_recv_packet(index: usize, packet: Packet);
}
