use crate::{alloc::string::ToString, REVISION_NUMBER};
use alloc::string::String;
use ibc::{
	core::{
		ics02_client::{
			client_type::ClientType as IbcClientType, error::Error as Ics02Error,
			height::Height as IbcHeight,
		},
		ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence},
		ics24_host::{
			error::ValidationError,
			identifier::{
				ChannelId as IbcChannelId, ClientId as IbcClientId,
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
use ibc::core::ics04_channel::timeout::TimeoutHeight as IbcTimeoutHeight;

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

/// ibc-rs' `PortId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PortId(pub Vec<u8>);

impl From<IbcPortId> for PortId {
	fn from(value: IbcPortId) -> Self {
		let value = value.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<PortId> for IbcPortId {
	fn from(value: PortId) -> Self {
		IbcPortId::from_str(&String::from_utf8(value.0).expect("convert from utf8 Error"))
			.expect("Never failed")
	}
}

/// ibc-rs' `ChannelId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ChannelId(pub Vec<u8>);

impl From<IbcChannelId> for ChannelId {
	fn from(value: IbcChannelId) -> Self {
		let value = value.to_string().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ChannelId> for IbcChannelId {
	fn from(value: ChannelId) -> Self {
		let value = String::from_utf8(value.0).expect("convert from utf8 Error");
		Self::from_str(&value).expect("convert channel id from str Error")
	}
}

/// ibc-rs' `TimeoutHeight` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum TimeoutHeight {
	Never,
	At(Height),
}

/// ibc-rs' `Height` representation in substrate
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
		IbcHeight::new(REVISION_NUMBER, height.revision_height).expect("Contruct IbcHeight Error")
	}
}

impl Height {
	pub fn new(revision_number: u64, revision_height: u64) -> Self {
		Self { revision_number, revision_height }
	}
}

pub const TENDERMINT_TYPE: &'static str = "07-tendermint";
pub const GRANDPA_TYPE: &'static str = "10-grandpa";

/// ibc-rs' `ClientType` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientType(Vec<u8>);

impl ClientType {
	pub fn new(s: &str) -> Self {
		let value = s.as_bytes().to_vec();
		Self(value)
	}

	pub fn to_string(&self) -> String {
		String::from_utf8(self.0.clone()).expect("Never failed")
	}
}

impl From<IbcClientType> for ClientType {
	fn from(value: IbcClientType) -> Self {
		Self::new(value.as_str())
	}
}

impl TryFrom<ClientType> for IbcClientType {
	type Error = Ics02Error;

	fn try_from(value: ClientType) -> Result<IbcClientType, Self::Error> {
		match value.to_string().as_str() {
			"07-tendermint" => Ok(IbcClientType::new(TENDERMINT_TYPE)),
			"10-grandpa" => Ok(IbcClientType::new(GRANDPA_TYPE)),
			unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
		}
	}
}

/// ibc-rs' `ClientId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientId(pub Vec<u8>);

impl From<IbcClientId> for ClientId {
	fn from(value: IbcClientId) -> Self {
		let value = value.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ClientId> for IbcClientId {
	fn from(value: ClientId) -> Self {
		let value = String::from_utf8(value.0).expect("convert from utf8 Error");
		IbcClientId::from_str(&value).expect("Never failed")
	}
}

/// ibc-rs' `ConnectionId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ConnectionId(pub Vec<u8>);

impl From<IbcConnectionId> for ConnectionId {
	fn from(value: IbcConnectionId) -> Self {
		let value = value.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ConnectionId> for IbcConnectionId {
	fn from(value: ConnectionId) -> Self {
		let value = String::from_utf8(value.0).expect("convert from utf8 Error");
		IbcConnectionId::from_str(&value).expect("Never failed")
	}
}

/// ibc-rs' `Timestamp` representation in substrate
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
	fn from(value: Timestamp) -> Self {
		let value = String::from_utf8(value.time).expect("convert from utf8 Error");
		Self::from_str(&value).expect("convert from str Error")
	}
}

/// ibc-rs' `Sequence` representation in substrate
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

/// ibc-rs' `Packet` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Packet {
	pub sequence: Sequence,
	pub source_port: PortId,
	pub source_channel: ChannelId,
	pub destination_port: PortId,
	pub destination_channel: ChannelId,
	pub data: Vec<u8>,
	pub timeout_height: TimeoutHeight,
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
				IbcTimeoutHeight::Never => TimeoutHeight::Never,
				IbcTimeoutHeight::At(value) =>
					TimeoutHeight::At(Height::new(value.revision_number(), value.revision_height())),
			},
			timeout_timestamp: val.timeout_timestamp.into(),
		}
	}
}

impl From<Packet> for IbcPacket {
	fn from(value: Packet) -> Self {
		Self {
			sequence: value.sequence.into(),
			source_port: value.source_port.into(),
			source_channel: value.source_channel.into(),
			destination_port: value.destination_port.into(),
			destination_channel: value.destination_channel.into(),
			data: value.data,
			timeout_height: match value.timeout_height {
				TimeoutHeight::Never => IbcTimeoutHeight::Never,
				TimeoutHeight::At(value) => IbcTimeoutHeight::At(
					IbcHeight::new(value.revision_number, value.revision_height).unwrap(),
				),
			},
			timeout_timestamp: value.timeout_timestamp.into(),
		}
	}
}
