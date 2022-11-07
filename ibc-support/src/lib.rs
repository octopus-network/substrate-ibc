#![cfg_attr(not(feature = "std"), no_std)]

pub mod ibc_trait;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// A trait handling asset ID and name
pub trait AssetIdAndNameProvider<AssetId> {
	type Err;

	fn try_get_asset_id(name: impl AsRef<[u8]>) -> Result<AssetId, Self::Err>;

	fn try_get_asset_name(asset_id: AssetId) -> Result<Vec<u8>, Self::Err>;
}

/// A struct corresponds to `Any` in crate "prost-types", used in ibc-rs.
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Any {
	pub type_url: Vec<u8>,
	pub value: Vec<u8>,
}

impl From<ibc_proto::google::protobuf::Any> for Any {
	fn from(any: ibc_proto::google::protobuf::Any) -> Self {
		Self { type_url: any.type_url.as_bytes().to_vec(), value: any.value }
	}
}
