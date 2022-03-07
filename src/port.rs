use super::*;

use crate::routing::Context;
use ibc::{
	core::ics05_port::{capabilities::Capability, context::PortReader, error::Error as ICS05Error},
	core::ics24_host::identifier::PortId,
};
use ibc::core::ics05_port::capabilities::CapabilityName;
use ibc::core::ics05_port::context::CapabilityReader;

impl<T: Config> CapabilityReader for Context<T> {
	fn get_capability(&self, _name: &CapabilityName) -> Result<Capability, ICS05Error> {
		todo!()
	}

	fn authenticate_capability(&self, _name: &CapabilityName, _capability: &Capability) -> Result<(), ICS05Error> {
		Ok(())
	}
}

impl<T: Config> PortReader for Context<T> {
	type ModuleId = ();

	fn lookup_module_by_port(
		&self,
		_port_id: &PortId,
	) -> Result<(Self::ModuleId, Capability), ICS05Error> {
		Ok(((), Capability::default()))
	}
}
