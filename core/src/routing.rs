use crate::BlockNumberFor;
use crate::{context::Context, Config};
pub use alloc::{
	format,
	string::{String, ToString},
};
use core::str::FromStr;
use ibc::core::router::module::Module;
use ibc::core::router::router::Router as RouterContext;
use ibc::core::router::types::module::ModuleId;
use ibc::{
	apps::transfer::types::{MODULE_ID_STR as TRANSFER_MODULE_ID, PORT_ID_STR as TRANSFER_PORT_ID},
	core::host::types::identifiers::PortId,
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

impl<T: Config> ibc::core::router::router::Router for Context<T>
where
	u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
{
	/// Returns a reference to a `Module` registered against the specified `ModuleId`
	fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
		self.router.0.get(module_id).map(Arc::as_ref)
	}

	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		match self.router.0.get_mut(module_id) {
			Some(arc_mod) => match Arc::get_mut(arc_mod) {
				Some(m) => Some(m),
				None => None,
			},
			None => None,
		}
	}

	/// Return the module_id associated with a given port_id
	fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
		self.lookup_module_by_port(port_id)
	}
}
