use ibc::core::ics26_routing::context::{Module, ModuleId};
use scale_info::prelude::{format, string::String};
use sp_std::{
	borrow::{Borrow, ToOwned},
	collections::btree_map::BTreeMap,
	fmt::{self, Debug},
	sync::Arc,
	vec,
};

pub trait AddModule {
	fn add_module(router: Router) -> Router;
}

pub struct DefaultRouter;

impl AddModule for DefaultRouter {
	fn add_module(router: Router) -> Router {
		router
	}
}

#[derive(Default, Clone)]
pub struct Router(pub BTreeMap<ModuleId, Arc<dyn Module>>);

impl Router {
	pub fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
		match self.0.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}
}

impl Debug for Router {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let keys = self
			.0
			.iter()
			.fold(vec![], |acc, (key, _)| [acc, vec![format!("{}", key)]].concat());

		write!(f, "Router(BTreeMap(key({:?})", keys.join(","))
	}
}

impl ibc::core::ics26_routing::context::Router for Router {
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
	}

	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		self.0.get(module_id.borrow()).is_some()
	}
}
