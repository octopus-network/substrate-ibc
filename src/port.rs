use super::*;

use crate::routing::Context;
use ibc::core::{
	ics05_port::{
		capabilities::{Capability, CapabilityName, PortCapability},
		context::{CapabilityReader, PortReader},
		error::Error as ICS05Error,
	},
	ics24_host::identifier::PortId,
	ics26_routing::context::ModuleId,
};

impl<T: Config> CapabilityReader for Context<T> {
	fn get_capability(&self, _name: &CapabilityName) -> Result<Capability, ICS05Error> {
		todo!()
	}

	fn authenticate_capability(
		&self,
		_name: &CapabilityName,
		_capability: &Capability,
	) -> Result<(), ICS05Error> {
		Ok(())
	}
}

impl<T: Config> PortReader for Context<T> {

	 /// Return the module_id along with the capability associated with a given port_id
	 fn lookup_module_by_port(&self, port_id: &PortId) -> Result<(ModuleId, PortCapability), ICS05Error> {
		
		todo!()
	}
}
