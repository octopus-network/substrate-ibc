use crate::Config;
use sp_std::marker::PhantomData;

use ibc::{applications::transfer::MODULE_ID_STR, core::ics26_routing::context::RouterBuilder};
use pallet_ics20_transfer::ics20_callback::IbcTransferModule;

/// A struct capturing all the functional dependencies (i.e., context)
/// which the ICS26 module requires to be able to dispatch and process IBC messages.
use crate::module::core::ics26_routing::{Router, SubstrateRouterBuilder};

#[derive(Clone, Debug)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: Router,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let r = SubstrateRouterBuilder::default()
			.add_route(MODULE_ID_STR.parse().unwrap(), IbcTransferModule(PhantomData::<T>)) // register transfer Module
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
