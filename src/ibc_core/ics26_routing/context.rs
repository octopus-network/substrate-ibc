use crate::{context::Context, utils::LOG_TARGET, *};
use ibc::core::ics26_routing::context::Ics26Context;
use log::trace;
use scale_info::prelude::{borrow::Borrow, collections::btree_map::BTreeMap};
use sp_std::sync::Arc;

use ibc::core::ics26_routing::context::{Module, ModuleId};

#[derive(Default, Clone)]
pub struct IBCRouter(pub BTreeMap<ModuleId, Arc<dyn Module>>);

/// A router maintains a mapping of `ModuleId`s against `Modules`. Implementations must not publicly
/// expose APIs to add new routes once constructed. Routes may only be added at the time of
/// instantiation using the `RouterBuilder`.
impl ibc::core::ics26_routing::context::Router for IBCRouter {
	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
	}

	/// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		self.0.get(module_id.borrow()).is_some()
	}
}

/// This trait captures all the functional dependencies (i.e., context) which the ICS26 module
/// requires to be able to dispatch and process IBC messages. In other words, this is the
/// representation of a chain from the perspective of the IBC module of that chain.
impl<T: Config> Ics26Context for Context<T> {
	type Router = IBCRouter;

	fn router(&self) -> &Self::Router {
		trace!(target: LOG_TARGET, "in routing: [route]");

		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		trace!(target: LOG_TARGET, "in routing: [router_mut]");

		&mut self.router
	}
}
