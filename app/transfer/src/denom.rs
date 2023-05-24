use alloc::vec::Vec;
use codec::{Decode, Encode};
use ibc::applications::transfer::denom::PrefixedDenom as IbcPrefixedDenom;

/// A type that contains the base denomination for ICS20 and the source tracing information path.
#[derive(Clone, Debug, Decode, Encode, scale_info::TypeInfo)]
pub struct PrefixedDenom {
	/// A series of `{port-id}/{channel-id}`s for tracing the source of the token.
	pub trace_path: Vec<u8>,
	/// Base denomination of the relayed fungible token.
	pub base_denom: Vec<u8>,
}

impl From<IbcPrefixedDenom> for PrefixedDenom {
	fn from(value: IbcPrefixedDenom) -> Self {
		Self {
			trace_path: alloc::format!("{}", value.trace_path).as_bytes().to_vec(),
			base_denom: value.base_denom.as_str().as_bytes().to_vec(),
		}
	}
}
