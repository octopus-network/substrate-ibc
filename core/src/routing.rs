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
		context::Router as RouterContext,
		ics24_host::identifier::PortId,
		ics26_routing::context::{Module, ModuleId},
	},
};
use sp_std::{
	borrow::{Borrow, ToOwned},
	collections::btree_map::BTreeMap,
	fmt::{self, Debug},
	sync::Arc,
	vec,
};

#[derive(Default)]
pub struct SubstrateRouterBuilder(Router);

impl SubstrateRouterBuilder {
	pub fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
		match self.0 .0.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}

	pub fn build(self) -> Router {
		self.0
	}
}

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

impl<T: Config> ibc::core::context::Router for Context<T> {
	fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
		self.router.0.get(module_id).map(Arc::as_ref)
	}
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		self.router.0.get_mut(module_id).and_then(Arc::get_mut)
	}

	fn has_route(&self, module_id: &ModuleId) -> bool {
		self.router.0.get(module_id).is_some()
	}

	fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
		match port_id.as_str() {
			TRANSFER_PORT_ID => Some(
				ModuleId::from_str(TRANSFER_MODULE_ID)
					.expect("Conver TRANSFER_MODULE_ID Never faild"),
			),
			_ => None,
		}
	}
}
