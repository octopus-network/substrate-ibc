use super::*;

use crate::routing::Context;
use ibc::{
	core::ics05_port::{capabilities::Capability, context::PortReader, error::Error as ICS05Error},
	core::ics24_host::identifier::PortId,
};
use ibc::core::ics05_port::capabilities::CapabilityName;
use ibc::core::ics05_port::context::CapabilityReader;

impl<T: Config> CapabilityReader for Context<T> {
	fn get_capability(&self, name: &CapabilityName) -> Result<Capability, ICS05Error> {
		todo!()
	}

	fn authenticate_capability(&self, name: &CapabilityName, capability: &Capability) -> Result<(), ICS05Error> {
		todo!()
	}
}

impl<T: Config> PortReader for Context<T> {
	type ModuleId = ();

	fn lookup_module_by_port(
		&self,
		port_id: &PortId,
	) -> Result<(Self::ModuleId, Capability), ICS05Error> {
		Ok(((), Capability::default()))
	}
}
