use super::header::AnyHeader;
use crate::module::core::ics24_host::{ClientId, ClientType, Height};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Attributes {
	pub height: Height,
	pub client_id: ClientId,
	pub client_type: ClientType,
	pub consensus_height: Height,
}

/// UpdateClient event signals a recent update of an on-chain client (IBC Client).
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UpdateClient {
	pub common: Attributes,
	pub header: Option<AnyHeader>,
}
