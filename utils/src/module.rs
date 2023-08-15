use ibc::core::router::{Module, ModuleId};
use scale_info::prelude::string::String;
use sp_std::{borrow::ToOwned, collections::btree_map::BTreeMap, fmt::Debug, sync::Arc};

pub trait AddModule {
	fn add_module(router: Router) -> Router;
}

pub struct DefaultRouter;

impl AddModule for DefaultRouter {
	fn add_module(router: Router) -> Router {
		router
	}
}

#[derive(Default, Clone, Debug)]
pub struct Router {
	pub router: BTreeMap<ModuleId, Arc<dyn Module>>,
}

impl Router {
	pub fn new() -> Self {
		Self { router: BTreeMap::new() }
	}

	pub fn add_route(
		mut self,
		module_id: ModuleId,
		module: impl Module + 'static,
	) -> Result<Self, String> {
		match self.router.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}
}
