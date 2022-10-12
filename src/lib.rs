#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unreachable_patterns)]

//! # Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to
//! interact with other chains in a trustees way via IBC protocol
//!
//! The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  
//! and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs),
//! which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).
extern crate alloc;
extern crate core;

pub use pallet::*;

use alloc::{
	format,
	string::{String, ToString},
};
use codec::{Decode, Encode};
use core::{marker::PhantomData, str::FromStr};
use scale_info::{prelude::vec, TypeInfo};
use core::fmt::Debug;
use frame_system::ensure_signed;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use ibc::core::ics24_host::identifier::ChannelId as IbcChannelId;
pub mod context;
pub mod events;
pub mod module;
pub mod utils;

use crate::context::Context;
use crate::module::core::ics24_host::{
	ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId,
};

pub const LOG_TARGET: &str = "runtime::pallet-ibc";
pub const REVISION_NUMBER: u64 = 0;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod type_define {
	use alloc::vec::Vec;

	pub type OctopusClientStatePath = Vec<u8>;
	pub type OctopusClientState = Vec<u8>;
	pub type OctopusClientId = Vec<u8>;
	pub type OctopusIbcHeight = Vec<u8>;
	pub type OctopusTimeStamp = Vec<u8>;
	pub type OctopusIbcHostHeight = Vec<u8>;
	pub type OctopusClientConsensusStatePath = Vec<u8>;
	pub type OctopusConsensusState = Vec<u8>;
	pub type OctopusConnectionsPath = Vec<u8>;
	pub type OctopusConnectionEnd = Vec<u8>;
	pub type OctopusChannelEndPath = Vec<u8>;
	pub type OctopusChannelEnd = Vec<u8>;
	pub type OctopusSeqSendsPath = Vec<u8>;
	pub type OctopusSeqRecvsPath = Vec<u8>;
	pub type OctopusSeqAcksPath = Vec<u8>;
	pub type OctopusAcksPath = Vec<u8>;
	pub type OctopusAcksHash = Vec<u8>;
	pub type OctopusClientTypePath = Vec<u8>;
	pub type OctopusClientType = Vec<u8>;
	pub type OctopusClientConnectionsPath = Vec<u8>;
	pub type OctopusConnectionId = Vec<u8>;
	pub type OctopusRecipientsPath = Vec<u8>;
	pub type OctopusRecipient = Vec<u8>;
	pub type OctopusCommitmentsPath = Vec<u8>;
	pub type OctopusCommitmentHash = Vec<u8>;
	pub type OctopusPortId = Vec<u8>;
	pub type OctopusChannelId = Vec<u8>;
	pub type OctopusSequence = u64;
	pub type OctopusWriteAckEvent = Vec<u8>;
	pub type PreviousHostHeight = u64;
}

#[frame_support::pallet]
pub mod pallet {
	use super::{type_define::*, *};
	use frame_support::{pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;
	use ibc::events::IbcEvent;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + Sync + Send + Debug + pallet_ics20_transfer::Config
	{
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The provider providing timestamp of host chain
		type TimeProvider: UnixTime;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// ClientStatePath(client_id) => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusClientStatePath, OctopusClientState, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientProcessedTimes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OctopusClientId,
		Blake2_128Concat,
		OctopusIbcHeight,
		OctopusTimeStamp,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientUpdateHeight<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OctopusClientId,
		Blake2_128Concat,
		OctopusIbcHeight,
		OctopusIbcHostHeight,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ClientConsensusStatePath(client_id, Height) => ConsensusState
	pub type ConsensusStates<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OctopusClientConsensusStatePath,
		OctopusConsensusState,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => ConnectionEnd
	pub type Connections<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusConnectionsPath, OctopusConnectionEnd, ValueQuery>;

	#[pallet::storage]
	/// ChannelEndPath(port_id, channel_id) => ChannelEnd
	pub type Channels<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusChannelEndPath, OctopusChannelEnd, ValueQuery>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => Vec<ChannelEndPath(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OctopusConnectionsPath,
		Vec<OctopusChannelEndPath>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// SeqSendsPath(port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusSeqSendsPath, OctopusSequence, ValueQuery>;

	#[pallet::storage]
	/// SeqRecvsPath(port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusSeqRecvsPath, OctopusSequence, ValueQuery>;

	#[pallet::storage]
	/// SeqAcksPath(port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusSeqAcksPath, OctopusSequence, ValueQuery>;

	#[pallet::storage]
	/// AcksPath(port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusAcksPath, OctopusAcksHash, ValueQuery>;

	#[pallet::storage]
	/// ClientTypePath(client_id) => client_type
	pub type Clients<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusClientTypePath, OctopusClientType, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn client_counter)]
	/// client counter
	pub type ClientCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn connection_counter)]
	/// connection counter
	pub type ConnectionCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// channel counter
	pub type ChannelCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// ClientConnectionsPath(client_id) => connection_id
	pub type ConnectionClient<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		OctopusClientConnectionsPath,
		OctopusConnectionId,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ReceiptsPath(port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusRecipientsPath, OctopusRecipient, ValueQuery>;

	#[pallet::storage]
	/// CommitmentsPath(port_id, channel_id, sequence) => hash of (timestamp, height, packet)
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, OctopusCommitmentsPath, OctopusCommitmentHash, ValueQuery>;

	#[pallet::storage]
	/// Previous host block height
	pub type OldHeight<T: Config> = StorageValue<_, PreviousHostHeight, ValueQuery>;

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IbcEvent { event: events::IbcEvent },
	}

	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Update the beefy light client failure!
		UpdateBeefyLightClientFailure,
		/// Receive mmr root block number less than client_state.latest_commitment.block_number
		ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber,
		/// Client id not found
		ClientIdNotFound,
		/// Encode error
		InvalidEncode,
		/// Decode Error
		InvalidDecode,
		/// FromUtf8Error
		InvalidFromUtf8,
		/// Invalid signed_commitment
		InvalidSignedCommitment,
		/// Empty latest_commitment
		EmptyLatestCommitment,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for most of the IBC request.
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		///
		/// The origin must be Signed and the sender must have sufficient funds fee.
		///
		/// Parameters:
		/// - `messages`: The arbitrary ICS message's representation in Substrate, which contains an
		///   URL and
		///  a serialized protocol buffer message. The URL name that uniquely identifies the type of
		/// the serialized protocol buffer message.
		///
		/// The relevant events are emitted when successful.
		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<ibc_support::Any>,
		) -> DispatchResultWithPostInfo {
			sp_tracing::within_span!(
			sp_tracing::Level::TRACE, "deliver";
			{
				let _sender = ensure_signed(origin)?;
				let mut ctx = Context::<T>::default();

				let messages: Vec<ibc_proto::google::protobuf::Any> = messages
					.into_iter()
					.map(|message| ibc_proto::google::protobuf::Any {
						type_url: String::from_utf8(message.type_url.clone()).unwrap(),
						value: message.value,
					})
					.collect();

				for (_, message) in messages.into_iter().enumerate() {

					match ibc::core::ics26_routing::handler::deliver(&mut ctx, message.clone()) {
						Ok(ibc::core::ics26_routing::handler::MsgReceipt { events, log: _log}) => {
							// deposit events about send packet event and ics20 transfer event
							for event in events {
								match event {
									IbcEvent::WriteAcknowledgement(ref write_ack) => {
										Self::deposit_event(event.clone().into());
									}
									_ => {
										Self::deposit_event(event.clone().into());
									}
								}
							}
						}
						Err(error) => {
							log::trace!(
								target: LOG_TARGET,
								"deliver error  : {:?} ",
								error
							);
						}
					};
				}

				Ok(().into())
			})
		}
	}
}
