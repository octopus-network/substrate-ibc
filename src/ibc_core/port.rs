use crate::*;
use log::{error, info, trace, warn};

use crate::context::Context;
use ibc::core::{
	ics05_port::{
		context::{PortReader},
		error::Error as Ics05Error,
	},
	ics24_host::identifier::PortId,
	ics26_routing::context::ModuleId,
};


impl<T: Config> PortReader for Context<T> {
	fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, Ics05Error> {
		todo!()
	}
}
