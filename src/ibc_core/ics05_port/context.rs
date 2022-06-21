use crate::*;
use log::trace;

use crate::context::Context;
use ibc::core::{
	ics05_port::{context::PortReader, error::Error as ICS05Error},
	ics24_host::identifier::PortId,
	ics26_routing::context::ModuleId,
};

impl<T: Config> PortReader for Context<T> {
	/// Return the module_id associated with a given port_id
	fn lookup_module_by_port(&self, _port_id: &PortId) -> Result<ModuleId, ICS05Error> {
		trace!(target:"runtime::pallet-ibc","in port: [lookup_module_by_port]");

		// todo
		let module_id = ModuleId::new("ibcmodule".to_string().into()).unwrap();
		Ok(module_id)
	}
}
