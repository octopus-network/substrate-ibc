#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

pub mod traits;
pub mod weights;
pub use weights::*;
pub mod module;

/// A trait handling asset ID and name
pub trait AssetIdAndNameProvider<AssetId> {
	type Err;

	fn try_get_asset_id(name: impl AsRef<[u8]>) -> Result<AssetId, Self::Err>;

	fn try_get_asset_name(asset_id: AssetId) -> Result<Vec<u8>, Self::Err>;
}
