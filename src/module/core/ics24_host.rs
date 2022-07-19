use crate::{alloc::string::ToString, from_channel_id_to_vec, REVISION_NUMBER};
use alloc::string::String;
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
use sp_std::{str::FromStr, vec::Vec};

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_runtime::RuntimeDebug;

use flex_error::{define_error, DisplayOnly, TraceError};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use tendermint_proto::Error as TendermintError;

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

impl TryFrom<PortId> for IbcPortId {
	type Error = Error;

	fn try_from(value: PortId) -> Result<Self, Self::Error> {
		let value = String::from_utf8(value.0).map_err(Error::invalid_from_utf8)?;
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

impl TryFrom<ChannelId> for IbcChannelId {
	type Error = Error;

	fn try_from(value: ChannelId) -> Result<Self, Self::Error> {
		let value = String::from_utf8(value.0).map_err(Error::invalid_from_utf8)?;
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
	fn from(ibc_height: IbcHeight) -> Self {
		Height::new(ibc_height.revision_number(), ibc_height.revision_height())
	}
}

impl From<Height> for IbcHeight {
	fn from(height: Height) -> Self {
		IbcHeight::new(REVISION_NUMBER, height.revision_height).unwrap()
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

impl TryFrom<Timestamp> for IbcTimestamp {
	type Error = Error;

	fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
		let value = String::from_utf8(value.time).map_err(Error::invalid_from_utf8)?;
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

impl From<Sequence> for IbcSequence {
	fn from(val: Sequence) -> Self {
		IbcSequence::from(val.0)
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
			timeout_height: match val.timeout_height {
				TimeoutHeight::Never => Height::new(REVISION_NUMBER, u64::MAX),
				TimeoutHeight::At(value) => value.into(),
			},
			timeout_timestamp: val.timeout_timestamp.into(),
		}
	}
}

impl TryFrom<Packet> for IbcPacket {
	type Error = Error;

	fn try_from(value: Packet) -> Result<Self, Self::Error> {
		Ok(IbcPacket {
			sequence: value.sequence.into(),
			source_port: value.source_port.try_into()?,
			source_channel: value.source_channel.try_into()?,
			destination_port: value.destination_port.try_into()?,
			destination_channel: value.destination_channel.try_into()?,
			data: value.data,
			timeout_height: TimeoutHeight::At(value.timeout_height.into()),
			timeout_timestamp: value.timeout_timestamp.try_into()?,
		})
	}
}
