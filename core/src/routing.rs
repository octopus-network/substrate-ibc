use crate::{context::Context, Config};
pub use alloc::{
	format,
	string::{String, ToString},
};
use core::str::FromStr;
use ibc::{
	applications::transfer::{
		MODULE_ID_STR as TRANSFER_MODULE_ID, PORT_ID_STR as TRANSFER_PORT_ID,
	},
	core::{
		ics24_host::identifier::PortId,
		router::{Module, ModuleId, Router as RouterContext},
	},
};
use sp_std::{
	borrow::{Borrow, ToOwned},
	collections::btree_map::BTreeMap,
	fmt::{self, Debug},
	sync::Arc,
	vec,
};

#[derive(Default, Clone)]
pub struct Router(pub BTreeMap<ModuleId, Arc<dyn Module>>);

impl Debug for Router {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let keys = self
			.0
			.iter()
			.fold(vec![], |acc, (key, _)| [acc, vec![format!("{}", key)]].concat());

		write!(f, "Router(BTreeMap(key({:?})", keys.join(","))
	}
}

impl<T: Config> ibc::core::router::Router for Context<T> {
	fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
		self.router.0.get(module_id).map(Arc::as_ref)
	}
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		// self.router.0.get_mut(module_id).and_then(Arc::get_mut)
		match self.router.0.get_mut(module_id) {
			Some(arc_mod) => match Arc::get_mut(arc_mod) {
				Some(m) => Some(m),
				None => None,
			},
			None => None,
		}
	}

	fn has_route(&self, module_id: &ModuleId) -> bool {
		self.router.0.get(module_id).is_some()
	}

	fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
		match port_id.as_str() {
			TRANSFER_PORT_ID => Some(ModuleId::new(TRANSFER_MODULE_ID.to_string())),
			_ => None,
		}
	}
}
