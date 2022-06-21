pub mod primitive {
	use crate::{alloc::string::ToString, from_channel_id_to_vec};
	use alloc::string::String;
	use ibc::{
		clients::ics10_grandpa::{
			client_state::ClientState as IbcClientState,
			help::{
				BlockHeader, Commitment, MmrRoot as IbcMmrRoot, SignedCommitment,
				ValidatorMerkleProof, ValidatorSet,
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
	use sp_std::{str::FromStr, vec::Vec};

	use codec::{Decode, Encode};
	use scale_info::TypeInfo;

	use sp_runtime::RuntimeDebug;

	use flex_error::{define_error, DisplayOnly};

	define_error! {
		#[derive(Debug, PartialEq, Eq)]
		Error {
			InvalidFromUtf8
				[DisplayOnly<alloc::string::FromUtf8Error>]
				| _ | { "invalid from utf8 error" },
			InvalidDecode
				[DisplayOnly<codec::Error>]
				| _ | { "invalid decode error" },
			ParseTimestampFailed
				[DisplayOnly<ibc::timestamp::ParseTimestampError>]
				| _ | { "invalid parse timestamp error" },
			ValidationFailed
				[DisplayOnly<ValidationError>]
				| _ | { "invalid validation error"},
			InvalidChainId
				[DisplayOnly<core::convert::Infallible>]
				|_| { "invalid chain id error" },
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct PortId(pub Vec<u8>);

	impl From<IbcPortId> for PortId {
		fn from(value: IbcPortId) -> Self {
			let value = value.0.as_bytes().to_vec();
			Self(value)
		}
	}

	impl PortId {
		pub fn to_ibc_port_id(self) -> Result<IbcPortId, Error> {
			let value = String::from_utf8(self.0).map_err(Error::invalid_from_utf8)?;
			Ok(IbcPortId(value))
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct ChannelId(pub Vec<u8>);

	impl From<IbcChannelId> for ChannelId {
		fn from(value: IbcChannelId) -> Self {
			let value = from_channel_id_to_vec(value);
			Self(value)
		}
	}

	impl ChannelId {
		pub fn to_ibc_channel_id(self) -> Result<IbcChannelId, Error> {
			let value = String::from_utf8(self.0).map_err(Error::invalid_from_utf8)?;
			IbcChannelId::from_str(&value).map_err(Error::validation_failed)
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
		pub fn new(revision_number: u64, revision_height: u64) -> Self {
			Self { revision_number, revision_height }
		}

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
		#[cfg(any(test, feature = "mocks"))]
		Mock = 9999,
	}

	impl From<IbcClientType> for ClientType {
		fn from(value: IbcClientType) -> Self {
			match value {
				IbcClientType::Tendermint => ClientType::Tendermint,
				IbcClientType::Grandpa => ClientType::Grandpa,
				#[cfg(any(test, feature = "mocks"))]
				IbcClientType::Mock => ClientType::Mock,
			}
		}
	}

	impl ClientType {
		pub fn to_ibc_client_type(self) -> IbcClientType {
			match self {
				ClientType::Tendermint => IbcClientType::Tendermint,
				ClientType::Grandpa => IbcClientType::Grandpa,
				#[cfg(any(test, feature = "mocks"))]
				ClientType::Mock => IbcClientType::Mock,
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
		pub fn to_ibc_client_id(self) -> Result<IbcClientId, Error> {
			let value = String::from_utf8(self.0).map_err(Error::invalid_from_utf8)?;
			Ok(IbcClientId(value))
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
		pub fn to_ibc_connection_id(self) -> Result<IbcConnectionId, Error> {
			let value = String::from_utf8(self.0).map_err(Error::invalid_from_utf8)?;
			Ok(IbcConnectionId(value))
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

	impl Timestamp {
		pub fn to_ibc_timestamp(self) -> Result<IbcTimestamp, Error> {
			let value = String::from_utf8(self.time).map_err(Error::invalid_from_utf8)?;
			IbcTimestamp::from_str(&value).map_err(Error::parse_timestamp_failed)
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
		pub fn to_ibc_packet(self) -> Result<IbcPacket, Error> {
			Ok(IbcPacket {
				sequence: self.sequence.to_ibc_sequence(),
				source_port: self.source_port.to_ibc_port_id()?,
				source_channel: self.source_channel.to_ibc_channel_id()?,
				destination_port: self.destination_port.to_ibc_port_id()?,
				destination_channel: self.destination_channel.to_ibc_channel_id()?,
				data: self.data,
				timeout_height: self.timeout_height.to_ibc_height(),
				timeout_timestamp: self.timeout_timestamp.to_ibc_timestamp()?,
			})
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
		pub fn to_ibc_mmr_root(self) -> Result<IbcMmrRoot, Error> {
			let decode_validator_proofs: Vec<ValidatorMerkleProof> = self
				.validator_merkle_proofs
				.into_iter()
				.map(|validator_proof| {
					ValidatorMerkleProof::decode(&mut &validator_proof[..]).unwrap() // TODO
				})
				.collect();
			Ok(IbcMmrRoot {
				block_header: BlockHeader::decode(&mut &self.block_header[..])
					.map_err(Error::invalid_decode)?,
				signed_commitment: SignedCommitment::decode(&mut &self.signed_commitment[..])
					.map_err(Error::invalid_decode)?,
				validator_merkle_proofs: decode_validator_proofs,
				mmr_leaf: self.mmr_leaf,
				mmr_leaf_proof: self.mmr_leaf_proof,
			})
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
		fn from(val: IbcClientState) -> Self {
			Self {
				chain_id: val.chain_id.as_str().as_bytes().to_vec(),
				block_number: val.block_number,
				frozen_height: val.frozen_height.map(|val| val.into()),
				block_header: BlockHeader::encode(&val.block_header),
				latest_commitment: Commitment::encode(&val.latest_commitment),
				validator_set: ValidatorSet::encode(&val.validator_set),
			}
		}
	}

	impl ClientState {
		pub fn to_ibc_client_state(self) -> Result<IbcClientState, Error> {
			let chain_id_str =
				String::from_utf8(self.chain_id).map_err(Error::invalid_from_utf8)?;
			Ok(IbcClientState {
				chain_id: IbcChainId::from_str(&chain_id_str).map_err(Error::invalid_chain_id)?,
				block_number: self.block_number,
				frozen_height: self.frozen_height.map(|value| value.to_ibc_height()),
				block_header: BlockHeader::decode(&mut &self.block_header[..])
					.map_err(Error::invalid_decode)?,
				latest_commitment: Commitment::decode(&mut &self.latest_commitment[..])
					.map_err(Error::invalid_decode)?,
				validator_set: ValidatorSet::decode(&mut &self.validator_set[..])
					.map_err(Error::invalid_decode)?,
			})
		}
	}
}
