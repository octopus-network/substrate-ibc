use super::*;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as ICS20Error, msgs::denom_trace::DenomTrace,
	},
	core::{
		ics04_channel::{
			channel::{Counterparty, Order},
			error::Error as Ics04Error,
			Version,
		},
		ics05_port::capabilities::ChannelCapability,
		ics24_host::identifier::{ChannelId, ConnectionId, PortId},
		ics26_routing::context::{Ics26Context, ModuleOutput, RouterBuilder},
	},
};

use crate::alloc::borrow::ToOwned;
use alloc::{
	borrow::{Borrow, Cow},
	collections::BTreeMap,
	sync::Arc,
};
use ibc::core::ics26_routing::context::{Module, ModuleId};
use scale_info::TypeInfo;

#[derive(Clone)]
pub struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	pub router: MockRouter,
}

impl<T: Config> Context<T> {
	pub fn new() -> Self {
		let r = MockRouterBuilder::default()
			.add_route("ibcmodule".parse().unwrap(), IbcModule::default())
			.unwrap()
			.build();

		Self { _pd: PhantomData::default(), router: r }
	}
}

#[derive(Debug, Default)]
struct IbcModule;

impl Module for IbcModule {
	fn on_chan_open_try(
		&mut self,
		_output: &mut ModuleOutput,
		_order: Order,
		_connection_hops: &[ConnectionId],
		_port_id: &PortId,
		_channel_id: &ChannelId,
		_channel_cap: &ChannelCapability,
		_counterparty: &Counterparty,
		_counterparty_version: &Version,
	) -> Result<Version, Ics04Error> {
		Ok(Version::ics20())
	}
}

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

#[derive(Debug, Default, Clone)]
pub struct MockRouter(BTreeMap<ModuleId, Arc<dyn Module>>);

impl ibc::core::ics26_routing::context::Router for MockRouter {
	fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
		log::trace!(target:"runtime::pallet-ibc","in routing: [get_route_mut]");

		self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
	}

	fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
		log::trace!(target:"runtime::pallet-ibc","in routing: [has_route]");
		self.0.get(module_id.borrow()).is_some()
	}
}

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

pub trait ModuleCallbacks {}
