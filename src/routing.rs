use super::*;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as ICS20Error, msgs::denom_trace::DenomTrace,
	},
	core::ics26_routing::context::Ics26Context,
};
// use ibc::core::ics26_routing::context::Ics26Context::Router;
use ibc::core::ics26_routing::context::ModuleId;
use alloc::borrow::{Borrow, Cow};
use ibc::core::ics26_routing::context::Module;

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

pub struct TempRouter;

impl ibc::core::ics26_routing::context::Router for TempRouter {
	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
    fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		todo!()
	}

    /// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
    fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		todo!()
	}
}

impl<T: Config> Ics26Context for Context<T> {
	type Router = TempRouter;

    fn router(&self) -> &Self::Router {
		todo!()
	}

    fn router_mut(&mut self) -> &mut Self::Router {
		todo!()
	}
}

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
