use crate::{
	applications::transfer::transfer_handler::IBCTransferModule,
	ibc_core::ics26_routing::context::IBCRouter, *,
};
use ibc::{
	applications::transfer::MODULE_ID_STR as TRANSFER_MODULE_ID,
	core::ics26_routing::context::{Module, ModuleId, RouterBuilder},
};
use scale_info::prelude::borrow::ToOwned;
use sp_std::sync::Arc;

#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: IBCRouter,
}

impl<T: Config> Context<T> {
	fn new() -> Self {
		let ibc_router = Self::default()
			.add_route(TRANSFER_MODULE_ID.parse().unwrap(), IBCTransferModule::default())
			.unwrap()
			.build();

		Self { _pd: PhantomData::default(), router: ibc_router }
	}
}

impl<T: Config> RouterBuilder for Context<T> {
	type Router = IBCRouter;

	fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
		match self.router.0.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}

	fn build(self) -> Self::Router {
		self.router
	}
}

/// default config ics20 transfer module
impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}
