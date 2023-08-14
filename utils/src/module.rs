use ibc::core::ics24_host::identifier::PortId;
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

impl ibc::core::router::Router for Router {
	/// Returns a reference to a `Module` registered against the specified `ModuleId`
	fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
		self.router.get(module_id).map(Arc::as_ref)
	}

	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		// NOTE: The following:

		// self.router.get_mut(module_id).and_then(Arc::get_mut)

		// doesn't work due to a compiler bug. So we expand it out manually.

		match self.router.get_mut(module_id) {
			Some(arc_mod) => match Arc::get_mut(arc_mod) {
				Some(m) => Some(m),
				None => None,
			},
			None => None,
		}
	}

	/// Return the module_id associated with a given port_id
	fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
		todo!()
	}
}
