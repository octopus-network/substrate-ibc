#![cfg_attr(not(feature = "std"), no_std)]
// todo need in future to remove
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(clippy::type_complexity)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::too_many_arguments)]

//! # Overview
//!
//! The goal of this pallet is to allow the blockchains built on Substrate to gain the ability to
//! interact with other chains in a trustees way via IBC protocol
//!
//! The pallet implements the chain specific logic of [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f),  
//! and is integrated with [ibc-rs](https://github.com/informalsystems/ibc-rs),
//! which implements the generic cross-chain logic in [ICS spec](https://github.com/cosmos/ibc/tree/ee71d0640c23ec4e05e924f52f557b5e06c1d82f).

extern crate alloc;

pub use pallet::*;

use alloc::{
	format,
	string::{String, ToString},
};
use core::{marker::PhantomData, str::FromStr};
use scale_info::{prelude::vec, TypeInfo};
use serde::{Deserialize, Serialize};

use beefy_light_client::commitment::{self, known_payload_ids::MMR_ROOT_ID};

use codec::{Codec, Decode, Encode};

use frame_support::{
	sp_runtime::traits::{AtLeast32BitUnsigned, CheckedConversion},
	sp_std::fmt::Debug,
	traits::{tokens::fungibles, Currency, ExistenceRequirement::AllowDeath},
	PalletId,
};
use frame_system::ensure_signed;
use sp_runtime::{traits::AccountIdConversion, DispatchError, RuntimeDebug, TypeId};
use sp_std::prelude::*;

use ibc::{
	applications::transfer::msgs::transfer::MsgTransfer,
	clients::ics10_grandpa::{
		client_state::ClientState,
		help::{self, BlockHeader, Commitment},
	},
	core::{
		ics02_client::{client_state::AnyClientState, height},
		ics04_channel::timeout::TimeoutHeight,
		ics24_host::identifier::{self, ChainId as ICS24ChainId, ChannelId as IbcChannelId},
		ics26_routing::msgs::Ics26Envelope,
	},
	timestamp,
	tx_msg::Msg,
};

use tendermint_proto::Protobuf;

pub mod context;
pub mod events;
pub mod module;
pub mod traits;
pub mod utils;

use crate::{context::Context, traits::AssetIdAndNameProvider};

use crate::module::{
	clients::ics10_grandpa::ClientState as EventClientState,
	core::ics24_host::{
		ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId, Timestamp,
	},
};

pub const LOG_TARGET: &str = "runtime::pallet-ibc";
pub const REVISION_NUMBER: u64 = 8888;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::module::{
		clients::ics10_grandpa::ClientState as EventClientState,
		core::ics24_host::{
			ChannelId, ClientId, ClientType, ConnectionId, Height, Packet, PortId, Sequence,
			Timestamp,
		},
	};
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			fungibles::{Inspect, Mutate, Transfer},
			UnixTime,
		},
	};
	use frame_system::pallet_prelude::*;
	use ibc::{
		applications::transfer::context::Ics20Context,
		clients::ics10_grandpa::consensus_state::ConsensusState as GPConsensusState,
		core::{
			ics02_client::client_consensus::AnyConsensusState,
			ics04_channel::{
				channel::{Counterparty, Order},
				context::ChannelKeeper,
				events::WriteAcknowledgement,
				Version,
			},
			ics24_host::{
				identifier::{ChannelId as IbcChannelId, PortId as IbcPortId},
				path::{ClientConsensusStatePath, ClientStatePath},
			},
			ics26_routing::error::Error as Ics26Error,
		},
		events::IbcEvent,
		signer::Signer,
	};
	use sp_runtime::traits::IdentifyAccount;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + core::marker::Sync + core::marker::Send {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type TimeProvider: UnixTime;

		/// Currency type of the runtime
		type Currency: Currency<Self::AccountId>;

		type AssetId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Codec
			+ Copy
			+ Debug
			+ Default
			+ MaybeSerializeDeserialize;

		type AssetBalance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ From<u128>
			+ Into<u128>
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug;

		type Assets: Transfer<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>;

		type AssetIdByName: AssetIdAndNameProvider<Self::AssetId>;

		/// Account Id Conversion from SS58 string or hex string
		type AccountIdConversion: TryFrom<Signer>
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ Clone;

		// config native token name
		const NATIVE_TOKEN_NAME: &'static [u8];
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// ClientStatePath(client_id) => ClientState
	pub type ClientStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (client_id, height) => timestamp
	pub type ClientProcessedTimes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (client_id, height) => host_height
	pub type ClientProcessedHeights<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		Blake2_128Concat,
		Vec<u8>,
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// ClientConsensusStatePath(client_id, Height) => ConsensusState
	pub type ConsensusStates<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => ConnectionEnd
	pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ChannelEndPath(port_id, channel_id) => ChannelEnd
	pub type Channels<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ConnectionsPath(connection_id) => Vec<ChannelEndPath(port_id, channel_id)>
	pub type ChannelsConnection<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	/// SeqSendsPath(port_id, channel_id) => sequence
	pub type NextSequenceSend<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// SeqRecvsPath(port_id, channel_id) => sequence
	pub type NextSequenceRecv<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// SeqAcksPath(port_id, channel_id) => sequence
	pub type NextSequenceAck<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

	#[pallet::storage]
	/// AcksPath(port_id, channel_id, sequence) => hash of acknowledgement
	pub type Acknowledgements<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ClientTypePath(client_id) => client_type
	pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

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
	pub type ConnectionClient<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// ReceiptsPath(port_id, channel_id, sequence) => receipt
	pub type PacketReceipt<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// CommitmentsPath(port_id, channel_id, sequence) => hash of (timestamp, height, packet)
	pub type PacketCommitment<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	/// (height, port_id, channel_id, sequence) => send-packet event
	pub type SendPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// (port_id, channel_id, sequence) => writ ack event
	pub type WriteAckPacketEvent<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, Vec<u8>>,
			NMapKey<Blake2_128Concat, u64>,
		),
		Vec<u8>,
		ValueQuery,
	>;

	#[pallet::storage]
	/// store latest height
	pub type LatestHeight<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	pub type OldHeight<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	/// key-value asset id with asset name
	pub type AssetIdByName<T: Config> =
		StorageMap<_, Twox64Concat, Vec<u8>, T::AssetId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub asset_id_by_name: Vec<(String, T::AssetId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { asset_id_by_name: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (token_id, id) in self.asset_id_by_name.iter() {
				<AssetIdByName<T>>::insert(token_id.as_bytes(), id);
			}
		}
	}

	/// Substrate IBC event list
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// emit update client state event
		UpdateClientState(Height, EventClientState),
		/// Raw Ibc events
		IbcEvents { events: Vec<events::IbcEvent> },
		/// emit escrow token
		EscrowToken(T::AccountId, T::AccountId, BalanceOf<T>),
		/// emit burn token
		BurnToken(T::AssetId, T::AccountId, T::AssetBalance),
		/// unescrow token
		UnEscrowToken(T::AccountId, T::AccountId, BalanceOf<T>),
		/// mint token
		MintToken(T::AssetId, T::AccountId, T::AssetBalance),
	}


	/// Errors in MMR verification informing users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// update the beefy light client failure!
		UpdateBeefyLightClientFailure,
		/// receive mmr root block number less than client_state.latest_commitment.block_number
		ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber,
		/// client id not found
		ClientIdNotFound,
		/// Encode error
		InvalidEncode,
		/// Decode Error
		InvalidDecode,
		/// FromUtf8Error
		InvalidFromUtf8,
		/// invalid signed_commitment
		InvalidSignedCommitment,
		/// empty latest_commitment
		EmptyLatestCommitment,
		/// invalid token id
		InvalidTokenId,
		/// wrong assert id
		WrongAssetId,
	}

	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsic", which are often compared to transactions.
	/// Dispatch able functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This function acts as an entry for all of the IBC request(except MMR root update).
		/// I.e., create clients, update clients, handshakes to create channels, ...etc
		#[pallet::weight(0)]
		pub fn deliver(
			origin: OriginFor<T>,
			messages: Vec<Any>,
			_tmp: u8,
		) -> DispatchResultWithPostInfo {
			sp_tracing::within_span!(
			sp_tracing::Level::TRACE, "deliver";
			{
				let _sender = ensure_signed(origin)?;
				let mut ctx = Context::<T>::new();

				let messages: Vec<ibc_proto::google::protobuf::Any> = messages
					.into_iter()
					.map(|message| ibc_proto::google::protobuf::Any {
						type_url: String::from_utf8(message.type_url.clone()).unwrap(),
						value: message.value,
					})
					.collect();

				log::trace!(target: LOG_TARGET, "received deliver : {:?} ", messages.iter().map(|message| message.type_url.clone()).collect::<Vec<_>>());

				let mut results: Vec<IbcEvent> = vec![];
				for (index, message) in messages.clone().into_iter().enumerate() {

					let mut result = Vec::new();
					match ibc::core::ics26_routing::handler::deliver(&mut ctx, message.clone()) {
						Ok(ibc::core::ics26_routing::handler::MsgReceipt { events, log: _log}) => {
							log::trace!(target: LOG_TARGET, "deliver event  : {:?} ", events);
							results.append(&mut result);

						}
						Err(error) => {
							log::trace!(target: LOG_TARGET, "deliver error  : {:?} ", error);
						}
					};

					log::info!("result: {:?}", result);

					results.append(&mut result);
				}

				// Self::deposit_event()

				Ok(().into())
			})
		}

		/// Update the MMR root stored in client_state
		/// Example of invoking this function via subxt
		#[pallet::weight(0)]
		pub fn update_client_state(
			origin: OriginFor<T>,
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			log::trace!(target: LOG_TARGET, "update_client_state: update_client_state request.");
			let _who = ensure_signed(origin)?;

			Self::inner_update_mmr_root(client_id, mmr_root)
		}

		#[pallet::weight(0)]
		pub fn delete_send_packet_event(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			log::trace!(target: LOG_TARGET, "delete_send_packet_event");

			let _who = ensure_signed(origin)?;
			<SendPacketEvent<T>>::drain();

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn delete_ack_packet_event(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			log::trace!(target: LOG_TARGET, "delete_ack_packet_event");

			let _who = ensure_signed(origin)?;
			<WriteAckPacketEvent<T>>::drain();

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// inner update mmr root
		fn inner_update_mmr_root(
			client_id: Vec<u8>,
			mmr_root: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			log::trace!(
				target: LOG_TARGET,
				"inner_update_client_state: update_client_state request."
			);

			// check if the client id exist?
			let client_id_str =
				String::from_utf8(client_id.clone()).map_err(|_| Error::<T>::InvalidFromUtf8)?;
			log::trace!(
				target: LOG_TARGET,
				"inner_update_client_state:  client id is {:?}",
				client_id_str
			);
			let ibc_client_id = identifier::ClientId::from_str(&client_id_str).unwrap();

			let decode_received_mmr_root =
				help::MmrRoot::decode(&mut &mmr_root[..]).map_err(|_| Error::<T>::InvalidDecode)?;
			log::trace!(
				target: LOG_TARGET,
				"inner_update_client_state:  decode mmr root is {:?}",
				decode_received_mmr_root
			);

			let mut client_state = ClientState::default();

			// read client state key, here is client state path
			let client_state_path =
				ClientStatePath(ibc_client_id.clone()).to_string().as_bytes().to_vec();
			if !<ClientStates<T>>::contains_key(client_state_path.clone()) {
				log::error!(
					"in inner_update_client_state: {:?} client_state not found !",
					client_id_str
				);

				return Err(Error::<T>::ClientIdNotFound.into())
			} else {
				// get client state from chain storage
				let data = <ClientStates<T>>::get(client_state_path.clone());
				let any_client_state =
					AnyClientState::decode_vec(&*data).map_err(|_| Error::<T>::InvalidDecode)?;
				client_state = match any_client_state {
					AnyClientState::Grandpa(value) => value,
					_ => unimplemented!(),
				};

				log::trace!(
					target: LOG_TARGET,
					"in inner_update_client_state: get client_state from chain storage: {:?}",
					client_state
				);
			}

			let signed_commitment =
				commitment::SignedCommitment::try_from(decode_received_mmr_root.signed_commitment)
					.map_err(|_| Error::<T>::InvalidSignedCommitment)?;

			let rev_block_number = signed_commitment.commitment.block_number;
			if rev_block_number <= client_state.latest_commitment.block_number {
				log::trace!(target: LOG_TARGET,"receive mmr root block number({}) less than client_state.latest_commitment.block_number({})",
				rev_block_number,client_state.latest_commitment.block_number);

				return Err(Error::<T>::ReceiveMmrRootBlockNumberLessThanClientStateLatestCommitmentBlockNumber.into());
			}
			// build new beefy light client by client_state
			let mut light_client = beefy_light_client::LightClient {
				latest_commitment: Some(client_state.latest_commitment.clone().into()),
				validator_set: client_state.validator_set.clone().into(),
				in_process_state: None,
			};
			log::trace!(
				target: LOG_TARGET,
				"build new beefy_light_client from client_state store in chain \n {:?}",
				light_client
			);

			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs = decode_received_mmr_root.validator_merkle_proofs;
			// covert the grandpa validator proofs to beefy_light_client::ValidatorMerkleProof
			let validator_proofs: Vec<beefy_light_client::ValidatorMerkleProof> = validator_proofs
				.into_iter()
				.map(|validator_proof| validator_proof.into())
				.collect();

			// encode signed_commitment
			let encoded_signed_commitment =
				commitment::SignedCommitment::encode(&signed_commitment);

			let mmr_leaf = decode_received_mmr_root.mmr_leaf;
			let mmr_leaf_proof = decode_received_mmr_root.mmr_leaf_proof;

			// verify mmr proof and update lc state
			let result = light_client.update_state(
				&encoded_signed_commitment,
				&validator_proofs,
				&mmr_leaf,
				&mmr_leaf_proof,
			);

			match result {
				Ok(_) => {
					log::trace!(target:"runtime::pallet-ibc","update the beefy light client success! and the beefy light client state is : {:?} \n",light_client);

					// update client_client block number and latest commitment
					let latest_commitment =
						light_client.latest_commitment.ok_or(Error::<T>::EmptyLatestCommitment)?;
					client_state.block_number = latest_commitment.block_number;
					client_state.latest_commitment = help::Commitment::from(latest_commitment);

					// update validator_set
					client_state.validator_set =
						help::ValidatorSet::from(light_client.validator_set.clone());

					// update block header
					client_state.block_header = decode_received_mmr_root.block_header;

					// save to chain
					let any_client_state = AnyClientState::Grandpa(client_state.clone());
					let data =
						any_client_state.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;
					// store client states key-value
					<ClientStates<T>>::insert(client_state_path.clone(), data);

					log::trace!(
						target: LOG_TARGET,
						"the updated client state is : {:?}",
						client_state
					);

					let mut consensus_state =
						GPConsensusState::new(client_state.block_header.clone());
					consensus_state.digest = client_state
						.latest_commitment
						.payload
						.get_raw(&MMR_ROOT_ID)
						.map(|value| value.clone())
						.unwrap_or_default();

					let any_consensus_state = AnyConsensusState::Grandpa(consensus_state);

					log::trace!(target: LOG_TARGET,"in ibc-lib : [store_consensus_state] >> client_id: {:?}, consensus_state = {:?}", client_id,  any_consensus_state);

					// store key
					let client_consensus_state_path = ClientConsensusStatePath {
						client_id: ibc_client_id.clone(),
						epoch: 8888,
						height: client_state.block_number as u64,
					}
					.to_string()
					.as_bytes()
					.to_vec();
					// store value
					let data =
						any_consensus_state.encode_vec().map_err(|_| Error::<T>::InvalidEncode)?;

					// if consensus state is empty insert a new item.
					<ConsensusStates<T>>::insert(client_consensus_state_path, data);

					// emit update state success event
					let event_height = Height {
						revision_number: 0,
						revision_height: client_state.block_number as u64,
					};
					let event_client_state = EventClientState::from(client_state);
					Self::deposit_event(Event::<T>::UpdateClientState(
						event_height,
						event_client_state,
					));
				},
				Err(e) => {
					log::error!(
						target: LOG_TARGET,
						"update the beefy light client failure! : {:?}",
						e
					);

					return Err(Error::<T>::UpdateBeefyLightClientFailure.into())
				},
			}

			Ok(().into())
		}
	}
}

fn store_send_packet<T: Config>(send_packet_event: &ibc::core::ics04_channel::events::SendPacket) {
	// store key port_id and channel_id
	let port_id = send_packet_event.packet.source_port.as_bytes().to_vec();
	let channel_id = from_channel_id_to_vec(send_packet_event.packet.source_channel.clone());

	// store value packet
	let packet =
		serde_json::to_string(&send_packet_event.packet.clone()).expect("serde packet error");

	log::trace!(
		target: LOG_TARGET,
		"in lib: [store_send_packet]. send_packet_event={:?}",
		send_packet_event
	);
	<SendPacketEvent<T>>::insert(
		(port_id, channel_id, u64::from(send_packet_event.packet.sequence)),
		packet.as_bytes(),
	);
}

fn store_write_ack<T: Config>(
	write_ack_event: &ibc::core::ics04_channel::events::WriteAcknowledgement,
) {
	use ibc::core::ics04_channel::events::WriteAcknowledgement;

	// store ack
	let port_id = write_ack_event.packet.source_port.as_bytes().to_vec();
	let channel_id = from_channel_id_to_vec(write_ack_event.packet.source_channel.clone());
	let sequence = u64::from(write_ack_event.packet.sequence);
	let write_ack = serde_json::to_string(&write_ack_event).expect("serde write ack event error");
	// store.Set((portID, channelID, sequence), WriteAckEvent)
	<WriteAckPacketEvent<T>>::insert((port_id, channel_id, sequence), write_ack.as_bytes());
}

impl<T: Config> AssetIdAndNameProvider<T::AssetId> for Pallet<T> {
	type Err = Error<T>;

	fn try_get_asset_id(name: impl AsRef<[u8]>) -> Result<<T as Config>::AssetId, Self::Err> {
		let asset_id = <AssetIdByName<T>>::try_get(name.as_ref().to_vec());
		match asset_id {
			Ok(id) => Ok(id),
			_ => Err(Error::<T>::InvalidTokenId),
		}
	}

	fn try_get_asset_name(asset_id: T::AssetId) -> Result<Vec<u8>, Self::Err> {
		let token_id = <AssetIdByName<T>>::iter().find(|p| p.1 == asset_id).map(|p| p.0);
		match token_id {
			Some(id) => Ok(id),
			_ => Err(Error::<T>::WrongAssetId),
		}
	}
}

pub fn from_channel_id_to_vec(value: IbcChannelId) -> Vec<u8> {
	value.to_string().as_bytes().to_vec()
}
