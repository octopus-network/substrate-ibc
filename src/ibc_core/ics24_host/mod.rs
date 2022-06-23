use scale_info::prelude::{
	string::{FromUtf8Error, String, ToString},
	vec::Vec,
};
use sp_std::{convert::Infallible, str::FromStr};

use codec::{Decode, Encode};
use ibc::{
	clients::ics10_grandpa::{
		client_state::ClientState as IbcClientState,
		help::{
			BlockHeader, Commitment, MmrRoot as IbcMmrRoot, SignedCommitment, ValidatorMerkleProof,
			ValidatorSet,
		},
	},
	core::{
		ics02_client::{client_type::ClientType as IbcClientType, height::Height as IbcHeight},
		ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence},
		ics24_host::{
			error::ValidationError,
			identifier::{
				ChainId as IbcChainId, ChannelId as IbcChannelId, ClientId as IbcClientId,
				ConnectionId as IbcConnectionId, PortId as IbcPortId,
			},
		},
	},
	timestamp::Timestamp as IbcTimestamp,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PortId(pub Vec<u8>);

impl From<IbcPortId> for PortId {
	fn from(value: IbcPortId) -> Self {
		let value = value.0.as_bytes().to_vec();
		Self(value)
	}
}

impl From<PortId> for IbcPortId {
	fn from(port_id: PortId) -> Self {
		let value = String::from_utf8(port_id.0).expect("Never fail");
		Self(value)
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ChannelId(pub Vec<u8>);

impl From<IbcChannelId> for ChannelId {
	fn from(value: IbcChannelId) -> Self {
		let value = value.to_string().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ChannelId> for IbcChannelId {
	fn from(channel_id: ChannelId) -> Self {
		let value = String::from_utf8(channel_id.0).expect("Never fail");
		Self::from_str(&value).expect("Never fail")
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Height {
	/// Previously known as "epoch"
	pub revision_number: u64,

	/// The height of a block
	pub revision_height: u64,
}

impl From<IbcHeight> for Height {
	fn from(IbcHeight { revision_number, revision_height }: IbcHeight) -> Self {
		Self { revision_number, revision_height }
	}
}

impl From<Height> for IbcHeight {
	fn from(height: Height) -> Self {
		Self { revision_number: height.revision_number, revision_height: height.revision_height }
	}
}

impl Height {
	pub fn new(revision_number: u64, revision_height: u64) -> Self {
		Self { revision_number, revision_height }
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ClientType {
	Tendermint,
	Grandpa,
	#[cfg(any(test, feature = "mocks"))]
	Mock = 9999,
}

impl From<IbcClientType> for ClientType {
	fn from(value: IbcClientType) -> Self {
		match value {
			IbcClientType::Tendermint => Self::Tendermint,
			IbcClientType::Grandpa => Self::Grandpa,
			#[cfg(any(test, feature = "mocks"))]
			IbcClientType::Mock => Self::Mock,
		}
	}
}

impl From<ClientType> for IbcClientType {
	fn from(client_type: ClientType) -> Self {
		match client_type {
			ClientType::Tendermint => Self::Tendermint,
			ClientType::Grandpa => Self::Grandpa,
			#[cfg(any(test, feature = "mocks"))]
			ClientType::Mock => Self::Mock,
		}
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientId(pub Vec<u8>);

impl From<IbcClientId> for ClientId {
	fn from(value: IbcClientId) -> Self {
		let value = value.0.as_bytes().to_vec();
		Self(value)
	}
}

impl From<ClientId> for IbcClientId {
	fn from(client_id: ClientId) -> Self {
		let value = String::from_utf8(client_id.0).expect("Never fail");
		Self(value)
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ConnectionId(pub Vec<u8>);

impl From<IbcConnectionId> for ConnectionId {
	fn from(value: IbcConnectionId) -> Self {
		let value = value.0.as_bytes().to_vec();
		Self(value)
	}
}

impl From<ConnectionId> for IbcConnectionId {
	fn from(connection_id: ConnectionId) -> Self {
		let value = String::from_utf8(connection_id.0).expect("Never fail");
		Self(value)
	}
}

/// Helper to convert between IBC timestamp and Substrate timestamp
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Timestamp {
	pub time: Vec<u8>,
}

impl From<IbcTimestamp> for Timestamp {
	fn from(val: IbcTimestamp) -> Self {
		Self { time: val.nanoseconds().to_string().as_bytes().to_vec() }
	}
}

impl From<Timestamp> for IbcTimestamp {
	fn from(time_stamp: Timestamp) -> Self {
		let value = String::from_utf8(time_stamp.time).expect("Never fail");
		Self::from_str(&value).expect("Never fail")
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Sequence(u64);

impl From<IbcSequence> for Sequence {
	fn from(val: IbcSequence) -> Self {
		Self(u64::from(val))
	}
}

impl From<Sequence> for IbcSequence {
	fn from(sequence: Sequence) -> Self {
		Self::from(sequence.0)
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MmrRoot {
	pub block_header: Vec<u8>,
	pub signed_commitment: Vec<u8>,
	pub validator_merkle_proofs: Vec<Vec<u8>>,
	pub mmr_leaf: Vec<u8>,
	pub mmr_leaf_proof: Vec<u8>,
}

impl From<IbcMmrRoot> for MmrRoot {
	fn from(ibc_mmr_root: IbcMmrRoot) -> Self {
		let encode_validator_proofs = ibc_mmr_root
			.validator_merkle_proofs
			.into_iter()
			.map(|validator_proof| ValidatorMerkleProof::encode(&validator_proof))
			.collect();

		Self {
			block_header: BlockHeader::encode(&ibc_mmr_root.block_header),
			signed_commitment: SignedCommitment::encode(&ibc_mmr_root.signed_commitment),
			validator_merkle_proofs: encode_validator_proofs,
			mmr_leaf: ibc_mmr_root.mmr_leaf,
			mmr_leaf_proof: ibc_mmr_root.mmr_leaf_proof,
		}
	}
}

impl From<MmrRoot> for IbcMmrRoot {
	fn from(mmr_root: MmrRoot) -> Self {
		let decode_validator_proofs: Vec<ValidatorMerkleProof> = mmr_root
			.validator_merkle_proofs
			.into_iter()
			.map(|validator_proof| {
				ValidatorMerkleProof::decode(&mut &validator_proof[..]).expect("Never fail")
			})
			.collect();

		Self {
			block_header: BlockHeader::decode(&mut &mmr_root.block_header[..]).expect("Never fail"),
			signed_commitment: SignedCommitment::decode(&mut &mmr_root.signed_commitment[..])
				.expect("Never fail"),
			validator_merkle_proofs: decode_validator_proofs,
			mmr_leaf: mmr_root.mmr_leaf,
			mmr_leaf_proof: mmr_root.mmr_leaf_proof,
		}
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientState {
	pub chain_id: Vec<u8>,
	/// block_number is height?
	pub block_number: u32,
	/// Block height when the client was frozen due to a misbehaviour
	pub frozen_height: Option<Height>,
	pub block_header: Vec<u8>,
	pub latest_commitment: Vec<u8>,
	pub validator_set: Vec<u8>,
}
impl From<IbcClientState> for ClientState {
	fn from(ibc_client_state: IbcClientState) -> Self {
		Self {
			chain_id: ibc_client_state.chain_id.as_str().as_bytes().to_vec(),
			block_number: ibc_client_state.block_number,
			frozen_height: ibc_client_state.frozen_height.map(|val| val.into()),
			block_header: BlockHeader::encode(&ibc_client_state.block_header),
			latest_commitment: Commitment::encode(&ibc_client_state.latest_commitment),
			validator_set: ValidatorSet::encode(&ibc_client_state.validator_set),
		}
	}
}

impl From<ClientState> for IbcClientState {
	fn from(client_state: ClientState) -> Self {
		let chain_id_str = String::from_utf8(client_state.chain_id).expect("Never fail");
		Self {
			chain_id: IbcChainId::from_str(&chain_id_str).expect("Never fail"),
			block_number: client_state.block_number,
			frozen_height: client_state.frozen_height.map(|value| value.into()),
			block_header: BlockHeader::decode(&mut &client_state.block_header[..])
				.expect("Never fail"),
			latest_commitment: Commitment::decode(&mut &client_state.latest_commitment[..])
				.expect("Never fail"),
			validator_set: ValidatorSet::decode(&mut &client_state.validator_set[..])
				.expect("Never fail"),
		}
	}
}
