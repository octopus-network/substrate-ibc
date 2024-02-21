use crate::{Config, Context};
pub use alloc::{
	format,
	string::{String, ToString},
};
use frame_system::pallet_prelude::BlockNumberFor;
use ibc::core::ics26_routing::context::{Module, ModuleId, RouterBuilder, RouterContext};
use pallet_ibc_utils::module::Router;
use sp_std::{borrow::ToOwned, sync::Arc};

#[derive(Default)]
pub struct SubstrateRouterBuilder(Router);

impl RouterBuilder for SubstrateRouterBuilder {
	type Router = Router;

	fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
		match self.0 .0.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}

	fn build(self) -> Self::Router {
		self.0
	}
}

impl<T: Config> RouterContext for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	type Router = Router;

	fn router(&self) -> &Self::Router {
		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		&mut self.router
	}
}
