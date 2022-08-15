use crate::{context::Context, *};
use alloc::{
	borrow::{Borrow, Cow, ToOwned},
	collections::BTreeMap,
	fmt::format,
	sync::Arc,
};
use core::fmt::Formatter;
use ibc::core::ics26_routing::context::{Ics26Context, Module, ModuleId, RouterBuilder};
use log::{error, info, trace, warn};
use scale_info::TypeInfo;

#[derive(Default)]
pub struct MockRouterBuilder(MockRouter);

impl RouterBuilder for MockRouterBuilder {
	type Router = MockRouter;

	fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
		match self.0 .0.insert(module_id, Arc::new(module)) {
			None => Ok(self),
			Some(_) => Err("Duplicate module_id".to_owned()),
		}
	}

	fn build(self) -> Self::Router {
		self.0
	}
}

#[derive(Default, Clone)]
pub struct MockRouter(BTreeMap<ModuleId, Arc<dyn Module>>);

impl Debug for MockRouter {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		let mut keys = vec![];
		for (key, _) in self.0.iter() {
			keys.push(format!("{}", key));
		}

		write!(f, "MockRouter(BTreeMap(key({:?})", keys.join(","))
	}
}

impl ibc::core::ics26_routing::context::Router for MockRouter {
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		trace!(target:"runtime::pallet-ibc","in routing: [get_route_mut]");

		self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
	}

	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		trace!(target:"runtime::pallet-ibc","in routing: [has_route]");
		self.0.get(module_id.borrow()).is_some()
	}
}

impl<T: Config> Ics26Context for Context<T> {
	type Router = MockRouter;

	fn router(&self) -> &Self::Router {
		trace!(target:"runtime::pallet-ibc","in routing: [route]");

		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		trace!(target:"runtime::pallet-ibc","in routing: [router_mut]");

		&mut self.router
	}
}
