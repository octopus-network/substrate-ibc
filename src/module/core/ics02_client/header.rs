use crate::module::clients::ics07_tendermint::header::Header as TendermintHeader;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[allow(clippy::large_enum_variant)]
pub enum AnyHeader {
	Tendermint(TendermintHeader),
	// Grandpa(GrandpaHeader),
}
