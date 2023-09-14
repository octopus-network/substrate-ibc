use alloc::{string::String, vec::Vec};
use codec::{Decode, Encode};
use core::str::FromStr;
use ibc::applications::transfer::{denom::PrefixedDenom as IbcPrefixedDenom, BaseDenom, TracePath};
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
		let trace_path_str = alloc::format!("{}", value.trace_path);
		let base_denom_str = value.base_denom.as_str();
		log::info!("🐙🐙 pallet_ics20_transfer::denom -> PrefixedDenom::From<IbcPrefixedDenom> for PrefixedDenom  trace_path_str:{:?} base_denom_str str:{:?}", trace_path_str,base_denom_str);
		Self {
			trace_path: trace_path_str.as_bytes().to_vec(),
			base_denom: base_denom_str.as_bytes().to_vec(),
		}
	}
}

impl From<PrefixedDenom> for IbcPrefixedDenom {
	fn from(value: PrefixedDenom) -> Self {
		let trace_path =
			TracePath::from_str(String::from_utf8(value.trace_path).unwrap().as_str()).unwrap();
		let base_denom =
			BaseDenom::from_str(String::from_utf8(value.base_denom).unwrap().as_str()).unwrap();

		log::info!("🐙🐙 pallet_ics20_transfer::denom -> From<PrefixedDenom> for IbcPrefixedDenom trace_path:{:?} base_denom:{:?}", trace_path,base_denom);
		Self { trace_path, base_denom }
	}
}
