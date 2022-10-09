use crate::{module::applications::transfer::transfer_handle_callback::TransferModule, *};
use ibc::{applications::transfer::MODULE_ID_STR, core::ics26_routing::context::RouterBuilder};

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
			.add_route(MODULE_ID_STR.parse().unwrap(), TransferModule(PhantomData::<T>)) // register transfer Module
			.unwrap()
			.build();

		Self { _pd: PhantomData::default(), router: r }
	}
}

/// default config ics20 transfer module
impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}
