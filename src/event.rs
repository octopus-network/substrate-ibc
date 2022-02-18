pub mod primitive {
	use crate::alloc::string::ToString;
	use alloc::string::String;
	use ibc::{
		core::ics02_client::{client_type::ClientType as IbcClientType, height::Height as IbcHeight},
		core::ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence},
		clients::ics10_grandpa::client_state::ClientState as IbcClientState,
		clients::ics10_grandpa::help::{
			BlockHeader, Commitment, MmrRoot as IbcMmrRoot, SignedCommitment, ValidatorMerkleProof,
			ValidatorSet,
		},
		core::ics24_host::identifier::{
			ChainId as IbcChainId, ChannelId as IbcChannelId, ClientId as IbcClientId,
			ConnectionId as IbcConnectionId, PortId as IbcPortId,
		},
		timestamp::Timestamp as IbcTimestamp,
	};
	use sp_std::{str::FromStr, vec::Vec};

	use codec::{Decode, Encode};
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

	impl PortId {
		pub fn to_ibc_port_id(self) -> IbcPortId {
			let value = String::from_utf8(self.0).unwrap();
			IbcPortId(value)
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct ChannelId(pub Vec<u8>);

	impl From<IbcChannelId> for ChannelId {
		fn from(value: IbcChannelId) -> Self {
			let value = value.0.as_bytes().to_vec();
			Self(value)
		}
	}

	impl ChannelId {
		pub fn to_ibc_channel_id(self) -> IbcChannelId {
			let value = String::from_utf8(self.0).unwrap();
			IbcChannelId(value)
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
			Height { revision_number, revision_height }
		}
	}

	impl Height {
		pub fn to_ibc_height(self) -> IbcHeight {
			IbcHeight {
				revision_number: self.revision_number,
				revision_height: self.revision_height,
			}
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum ClientType {
		Tendermint,
		Grandpa,
	}

	impl From<IbcClientType> for ClientType {
		fn from(value: IbcClientType) -> Self {
			match value {
				IbcClientType::Tendermint => ClientType::Tendermint,
				IbcClientType::Grandpa => ClientType::Grandpa,
				_ => unreachable!(),
			}
		}
	}

	impl ClientType {
		pub fn to_ibc_client_type(self) -> IbcClientType {
			match self {
				ClientType::Tendermint => IbcClientType::Tendermint,
				ClientType::Grandpa => IbcClientType::Grandpa,
				_ => unreachable!(),
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

	impl ClientId {
		pub fn to_ibc_client_id(self) -> IbcClientId {
			let value = String::from_utf8(self.0).unwrap();
			IbcClientId(value)
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

	impl ConnectionId {
		pub fn to_ibc_connection_id(self) -> IbcConnectionId {
			let value = String::from_utf8(self.0).unwrap();
			IbcConnectionId(value)
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Timestamp {
		pub time: Vec<u8>,
	}

	impl From<IbcTimestamp> for Timestamp {
		fn from(val: IbcTimestamp) -> Self {
			Self { time: val.as_nanoseconds().to_string().as_bytes().to_vec() }
		}
	}

	impl Timestamp {
		pub fn to_ibc_timestamp(self) -> IbcTimestamp {
			let value = String::from_utf8(self.time).unwrap();
			IbcTimestamp::from_str(&value).unwrap()
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Sequence(u64);

	impl From<IbcSequence> for Sequence {
		fn from(val: IbcSequence) -> Self {
			Self(u64::from(val))
		}
	}

	impl Sequence {
		pub fn to_ibc_sequence(self) -> IbcSequence {
			IbcSequence::from(self.0)
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Packet {
		pub sequence: Sequence,
		pub source_port: PortId,
		pub source_channel: ChannelId,
		pub destination_port: PortId,
		pub destination_channel: ChannelId,
		pub data: Vec<u8>,
		pub timeout_height: Height,
		pub timeout_timestamp: Timestamp,
	}

	impl From<IbcPacket> for Packet {
		fn from(val: IbcPacket) -> Self {
			Self {
				sequence: val.sequence.into(),
				source_port: val.source_port.into(),
				source_channel: val.source_channel.into(),
				destination_port: val.destination_port.into(),
				destination_channel: val.destination_channel.into(),
				data: val.data,
				timeout_height: val.timeout_height.into(),
				timeout_timestamp: val.timeout_timestamp.into(),
			}
		}
	}

	impl Packet {
		pub fn to_ibc_packet(self) -> IbcPacket {
			IbcPacket {
				sequence: self.sequence.to_ibc_sequence(),
				source_port: self.source_port.to_ibc_port_id(),
				source_channel: self.source_channel.to_ibc_channel_id(),
				destination_port: self.destination_port.to_ibc_port_id(),
				destination_channel: self.destination_channel.to_ibc_channel_id(),
				data: self.data,
				timeout_height: self.timeout_height.to_ibc_height(),
				timeout_timestamp: self.timeout_timestamp.to_ibc_timestamp(),
			}
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
		fn from(val: IbcMmrRoot) -> Self {
			let encode_validator_proofs = val
				.validator_merkle_proofs
				.into_iter()
				.map(|validator_proof| ValidatorMerkleProof::encode(&validator_proof))
				.collect();

			Self {
				block_header: BlockHeader::encode(&val.block_header),
				signed_commitment: SignedCommitment::encode(&val.signed_commitment),
				validator_merkle_proofs: encode_validator_proofs,
				mmr_leaf: val.mmr_leaf,
				mmr_leaf_proof: val.mmr_leaf_proof,
			}
		}
	}

	impl MmrRoot {
		pub fn to_ibc_mmr_root(self) -> IbcMmrRoot {
			let decode_validator_proofs: Vec<ValidatorMerkleProof> = self
				.validator_merkle_proofs
				.into_iter()
				.map(|validator_proof| {
					ValidatorMerkleProof::decode(&mut &validator_proof[..]).unwrap()
				})
				.collect();
			IbcMmrRoot {
				block_header: BlockHeader::decode(&mut &self.block_header[..]).unwrap(),
				signed_commitment: SignedCommitment::decode(&mut &self.signed_commitment[..])
					.unwrap(),
				validator_merkle_proofs: decode_validator_proofs,
				mmr_leaf: self.mmr_leaf,
				mmr_leaf_proof: self.mmr_leaf_proof,
			}
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct ClientState {
		pub chain_id: Vec<u8>,
		/// block_number is height?
		pub block_number: u32,
		/// Block height when the client was frozen due to a misbehaviour
		pub frozen_height: Height,
		pub block_header: Vec<u8>,
		pub latest_commitment: Vec<u8>,
		pub validator_set: Vec<u8>,
	}
	impl From<IbcClientState> for ClientState {
		fn from(val: IbcClientState) -> Self {
			Self {
				chain_id: val.chain_id.as_str().as_bytes().to_vec(),
				// chain_id: val.chain_id,
				block_number: val.block_number,
				frozen_height: Height::from(val.frozen_height),
				block_header: BlockHeader::encode(&val.block_header),
				latest_commitment: Commitment::encode(&val.latest_commitment),
				validator_set: ValidatorSet::encode(&val.validator_set),
			}
		}
	}

	impl ClientState {
		pub fn to_ibc_client_state(self) -> IbcClientState {
			let chain_id_str = String::from_utf8(self.chain_id).unwrap();
			IbcClientState {
				chain_id: IbcChainId::from_str(&chain_id_str).unwrap(),
				// chain_id: self.chain_id,
				block_number: self.block_number,
				frozen_height: self.frozen_height.to_ibc_height(),
				block_header: BlockHeader::decode(&mut &self.block_header[..]).unwrap(),
				latest_commitment: Commitment::decode(&mut &self.latest_commitment[..]).unwrap(),
				validator_set: ValidatorSet::decode(&mut &self.validator_set[..]).unwrap(),
			}
		}
	}
}
