use crate::*;
use alloc::{
	borrow::{Borrow, Cow, ToOwned},
	collections::BTreeMap,
	sync::Arc,
};
use scale_info::TypeInfo;

use crate::module::applications::transfer::transfer_handle_callback::TransferModule;
use ibc::{
	applications::transfer::{context::Ics20Context, error::Error as ICS20Error, MODULE_ID_STR},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			Version,
		},
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{
			Ics26Context, Module, ModuleId, ModuleOutputBuilder, RouterBuilder,
		},
	},
};

use crate::module::core::ics26_routing::{MockRouter, MockRouterBuilder};
#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: MockRouter,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let r = MockRouterBuilder::default()
			.add_route(MODULE_ID_STR.parse().unwrap(), TransferModule(PhantomData::<T>)) // register transfer Module
			.unwrap()
			.build();

		Self { _pd: PhantomData::default(), router: r }
	}
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}
