use ibc::applications::transfer::MODULE_ID_STR as IBC_TRANSFER_MODULE_ID;
use ibc::core::RouterError;
use ibc::core::{
	ics24_host::identifier::PortId,
	router::{Module, ModuleId},
};
use scale_info::prelude::string::String;
use sp_std::{borrow::ToOwned, collections::btree_map::BTreeMap, fmt::Debug, sync::Arc};

use crate::app::transfer::IbcTransferModule;
use crate::Config;

pub mod dynamic_typing {
	use core::any::Any;

	pub trait AsAny: Any {
		fn as_any(&self) -> &dyn Any;
	}

	impl<T: Any> AsAny for T {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}
}

use dynamic_typing::AsAny;

pub trait IbcModule: AsAny + Module {}

#[derive(Default, Clone, Debug)]
pub struct Router {
	/// Mapping of which IBC modules own which port
	pub port_to_module_map: BTreeMap<PortId, ModuleId>,
	/// Mapping of module ids to the module itself
	pub router: BTreeMap<ModuleId, Arc<dyn Module>>,
}

impl Router {
	pub fn new<T: Config>() -> Self {
		let mut port_to_module_map = BTreeMap::default();
		let transfer_module_id: ModuleId = ModuleId::new(IBC_TRANSFER_MODULE_ID.to_string());
		port_to_module_map.insert(PortId::transfer(), transfer_module_id.clone());
		let transfer_module = IbcTransferModule::<T>::new();
		let router = Self { router: BTreeMap::default(), port_to_module_map };
		router
			.add_module(PortId::transfer(), transfer_module_id, transfer_module)
			.expect("build Router error")
	}

	pub fn add_module(
		mut self,
		port_id: PortId,
		module_id: ModuleId,
		module: impl Module + IbcModule + 'static,
	) -> Result<Self, String> {
		match (
			self.port_to_module_map.insert(port_id, module_id.clone()),
			self.router.insert(module_id, Arc::new(module)),
		) {
			(None, None) => Ok(self),
			(_, _) => return Err("Duplicate module id or port id".to_owned()),
		}
	}

	// pub fn get_transfer_module<T: Config>(
	// 	&'static self,
	// ) -> Result<IbcTransferModule<T>, RouterError> {
	// 	use ibc::core::router::Router;
	// 	let module_id = ModuleId::new(IBC_TRANSFER_MODULE_ID.to_string());
	// 	let transfer_module = self
	// 		.get_route(&module_id)
	// 		.ok_or_else(|| RouterError::ModuleNotFound)?
	// 		.clone()
	// 		.as_any()
	// 		.downcast_ref::<IbcTransferModule<T>>()
	// 		.ok_or_else(|| RouterError::ModuleNotFound)?
	// 		.clone();

	// 	Ok(transfer_module)
	// }
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
		self.port_to_module_map
			.get(port_id)
			.ok_or(RouterError::UnknownPort { port_id: port_id.clone() })
			.map(Clone::clone)
			.ok()
	}
}
