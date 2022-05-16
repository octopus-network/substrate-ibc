use crate::{context::Context, *};

use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::Ics20Context, error::Error as Ics20Error, msgs::denom_trace::DenomTrace,
	},
	core::ics24_host::identifier::PortId,
};

impl<T: Config> Ics20Context for Context<T> {
	// GetDenomTrace retreives the full identifiers trace and base denomination from the store.
	fn get_denom_trace(&self, denom_trace_hash: &[u8]) -> Result<DenomTrace, Ics20Error> {
		log::trace!(target:"runtime::pallet-ibc","in transfer : [denom trace hash]");

		if <Denomination<T>>::contains_key(denom_trace_hash) {
			let data = <Denomination<T>>::get(denom_trace_hash);
			let denom_trace = DenomTrace::decode_vec(&data).map_err(Ics20Error::invalid_decode)?;
			log::trace!(target:"runtime::pallet-ibc","in transfer : [denom trace] >> {:?}", denom_trace);
			Ok(denom_trace)
		} else {
			log::trace!(target:"runtime::pallet-ibc","in transfer : [denom trace] >> denom trace not found");
			Err(Ics20Error::denom_trace_not_found(String::from("denom trace not found")))
		}
	}

	// HasDenomTrace checks if a the key with the given denomination trace hash exists on the store.
	fn has_denom_trace(&self, denom_trace_hash: &[u8]) -> bool {
		log::trace!(target:"runtime::pallet-ibc","in transfer : [denom trace hash]");

		<Denomination<T>>::contains_key(denom_trace_hash)
	}

	// SetDenomTrace sets a new {trace hash -> denom trace} pair to the store.
	fn set_denom_trace(&self, denom_trace: &DenomTrace) -> Result<(), Ics20Error> {
		log::trace!(target:"runtime::pallet-ibc","in transfer : [denom trace]");

		let data = denom_trace.encode_vec().map_err(Ics20Error::invalid_encode)?;

		<Denomination<T>>::insert(denom_trace.hash(), data);
		Ok(())
	}

	// todo
	fn get_port(&self) -> Result<PortId, Ics20Error> {
		Ok(PortId::transfer())
	}
}