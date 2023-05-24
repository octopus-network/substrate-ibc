use crate::Config;
use alloc::{borrow::ToOwned, string::String, sync::Arc};
use sp_std::marker::PhantomData;

use ibc::core::router::{Module, ModuleId};
use ibc_support::module::{AddModule, Router};

#[derive(Clone, Debug)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: Router,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let router = Router::new();
		let r = T::IbcModule::add_module(router);
		Self { _pd: PhantomData::default(), router: r }
	}

	pub fn add_route(
		&mut self,
		module_id: ModuleId,
		module: impl Module + 'static,
	) -> Result<(), String> {
		match self.router.router.insert(module_id, Arc::new(module)) {
			None => Ok(()),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}
}

impl<T: Config> Default for Context<T> {
	fn default() -> Self {
		Self::new()
	}
}
