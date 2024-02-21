#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_system::pallet_prelude::*;
use ibc_proto::google::protobuf::Any;
use scale_info::prelude::vec;
use sp_std::{fmt::Debug, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config + pallet_ibc::Config {
		/// The aggregated event type of the runtime.
		type RuntimeEvent: Parameter
			+ Member
			+ From<Event<Self>>
			+ Debug
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u64: From<<T as pallet_timestamp::Config>::Moment> + From<BlockNumberFor<T>>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn dispatch(origin: OriginFor<T>, messages: Vec<Any>) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;

			for message in messages {
				match message.type_url.as_str() {
					"/ibc.applications.transfer.v1.MsgTransfer" => {
						#[cfg(feature = "ics20")]
						{
							<pallet_ics20::Pallet<T> as pallet_ibc_utils::Router>::dispatch(vec![
								message,
							])?;
						}
					},
					_ => {
						<pallet_ibc::Pallet<T> as pallet_ibc_utils::Router>::dispatch(vec![
							message,
						])?;
					},
				}
			}

			Ok(().into())
		}
	}
}
