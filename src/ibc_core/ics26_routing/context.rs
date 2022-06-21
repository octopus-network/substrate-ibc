use crate::{context::Context, *};
use alloc::{borrow::Borrow, collections::BTreeMap, sync::Arc};
use ibc::core::ics26_routing::context::Ics26Context;
use log::trace;

use ibc::core::ics26_routing::context::{Module, ModuleId};

#[derive(Debug, Default, Clone)]
pub struct IBCRouter(pub BTreeMap<ModuleId, Arc<dyn Module>>);

impl ibc::core::ics26_routing::context::Router for IBCRouter {
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
	}

	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		self.0.get(module_id.borrow()).is_some()
	}
}

impl<T: Config> Ics26Context for Context<T> {
	type Router = IBCRouter;

	fn router(&self) -> &Self::Router {
		trace!(target:"runtime::pallet-ibc","in routing: [route]");

		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		trace!(target:"runtime::pallet-ibc","in routing: [router_mut]");

		&mut self.router
	}
}
