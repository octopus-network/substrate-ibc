use super::*;

use crate::routing::Context;
use ibc::ics05_port::capabilities::Capability;
use ibc::ics05_port::context::PortReader;
use ibc::ics24_host::identifier::PortId;

impl<T: Config> PortReader for Context<T> {
	fn lookup_module_by_port(&self, _port_id: &PortId) -> Option<Capability> {
		log::info!("in port: [look_module_by_port]");

		Some(Capability::default())
	}
	fn authenticate(&self, _key: &Capability, _port_id: &PortId) -> bool {
		log::info!("in port: [authenticate]");
		true
	}
}
