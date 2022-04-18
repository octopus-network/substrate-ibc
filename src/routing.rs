use super::*;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as ICS20Error, msgs::denom_trace::DenomTrace,
	},
	core::ics26_routing::context::Ics26Context,
};
// use ibc::core::ics26_routing::context::Ics26Context::Router;
use alloc::borrow::{Borrow, Cow};
use ibc::core::ics26_routing::context::{Module, ModuleId};

#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub tmp: u8,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		Self { _pd: PhantomData::default(), tmp: 0 }
	}
}

pub struct TempRouter;

impl ibc::core::ics26_routing::context::Router for TempRouter {
	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		todo!()
	}

	/// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		todo!()
	}
}

impl<T: Config> Ics26Context for Context<T> {
	type Router = TempRouter;

	fn router(&self) -> &Self::Router {
		todo!()
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		todo!()
	}
}

pub trait ModuleCallbacks {}
