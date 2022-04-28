use crate::{
	context::{Context, MockRouter},
	*,
};
use ibc::core::ics26_routing::context::Ics26Context;

impl<T: Config> Ics26Context for Context<T> {
	type Router = MockRouter;

	fn router(&self) -> &Self::Router {
		log::trace!(target:"runtime::pallet-ibc","in routing: [route]");
		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		log::trace!(target:"runtime::pallet-ibc","in routing: [router_mut]");
		&mut self.router
	}
}
