use crate::{
	prelude::{String, ToString},
	REVISION_NUMBER,
};
use ibc::{
	core::{
		ics02_client::{
			client_type::ClientType as IbcClientType, error::Error as Ics02Error,
			height::Height as IbcHeight,
		},
		ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence},
		ics24_host::identifier::{
			ChannelId as IbcChannelId, ClientId as IbcClientId, ConnectionId as IbcConnectionId,
			PortId as IbcPortId,
		},
	},
	timestamp::Timestamp as IbcTimestamp,
};
use sp_std::{str::FromStr, vec::Vec};

use codec::{Decode, Encode};
use ibc::core::ics04_channel::timeout::TimeoutHeight as IbcTimeoutHeight;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub const TENDERMINT_TYPE: &'static str = "07-tendermint";

/// ibc-rs' `PortId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PortId(pub Vec<u8>);

impl From<IbcPortId> for PortId {
	fn from(ibc_port_id: IbcPortId) -> Self {
		let value = ibc_port_id.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<PortId> for IbcPortId {
	fn from(port_id: PortId) -> Self {
		IbcPortId::from_str(
			&String::from_utf8(port_id.0).expect("hex-encoded string should always be valid UTF-8"),
		)
		.expect("Never failed")
	}
}

/// ibc-rs' `ChannelId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ChannelId(pub Vec<u8>);

impl From<IbcChannelId> for ChannelId {
	fn from(ibc_channel_id: IbcChannelId) -> Self {
		let value = ibc_channel_id.to_string().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ChannelId> for IbcChannelId {
	fn from(channel_id: ChannelId) -> Self {
		let value = String::from_utf8(channel_id.0)
			.expect("hex-encoded string should always be valid UTF-8");
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

/// ibc-rs' `ClientType` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientType(Vec<u8>);

impl ClientType {
	pub fn new(s: &str) -> Self {
		let value = s.as_bytes().to_vec();
		Self(value)
	}

	pub fn to_string(&self) -> String {
		String::from_utf8(self.0.clone()).expect("hex-encoded string should always be valid UTF-8")
	}
}

impl From<IbcClientType> for ClientType {
	fn from(ibc_client_type: IbcClientType) -> Self {
		Self::new(ibc_client_type.as_str())
	}
}

impl TryFrom<ClientType> for IbcClientType {
	type Error = Ics02Error;

	fn try_from(client_type: ClientType) -> Result<IbcClientType, Self::Error> {
		match client_type.to_string().as_str() {
			"07-tendermint" => Ok(IbcClientType::new(TENDERMINT_TYPE)),
			unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
		}
	}
}

/// ibc-rs' `ClientId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientId(pub Vec<u8>);

impl From<IbcClientId> for ClientId {
	fn from(ibc_client_id: IbcClientId) -> Self {
		let value = ibc_client_id.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ClientId> for IbcClientId {
	fn from(client_id: ClientId) -> Self {
		let value = String::from_utf8(client_id.0)
			.expect("hex-encoded string should always be valid UTF-8");
		IbcClientId::from_str(&value).expect("Never failed")
	}
}

/// ibc-rs' `ConnectionId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ConnectionId(pub Vec<u8>);

impl From<IbcConnectionId> for ConnectionId {
	fn from(ibc_connection_id: IbcConnectionId) -> Self {
		let value = ibc_connection_id.as_str().as_bytes().to_vec();
		Self(value)
	}
}

impl From<ConnectionId> for IbcConnectionId {
	fn from(connection_id: ConnectionId) -> Self {
		let value = String::from_utf8(connection_id.0)
			.expect("hex-encoded string should always be valid UTF-8");
		IbcConnectionId::from_str(&value).expect("Never failed")
	}
}

/// ibc-rs' `Timestamp` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Timestamp {
	pub time: Vec<u8>,
}

impl From<IbcTimestamp> for Timestamp {
	fn from(ibc_timestamp: IbcTimestamp) -> Self {
		Self { time: ibc_timestamp.nanoseconds().to_string().as_bytes().to_vec() }
	}
}

impl From<Timestamp> for IbcTimestamp {
	fn from(timestamp: Timestamp) -> Self {
		let value = String::from_utf8(timestamp.time)
			.expect("hex-encoded string should always be valid UTF-8");
		Self::from_str(&value).expect("convert from str Error")
	}
}

/// ibc-rs' `Sequence` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Sequence(u64);

impl From<IbcSequence> for Sequence {
	fn from(ibc_sequence: IbcSequence) -> Self {
		Self(u64::from(ibc_sequence))
	}
}

impl From<Sequence> for IbcSequence {
	fn from(sequence: Sequence) -> Self {
		IbcSequence::from(sequence.0)
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
	fn from(ibc_packet: IbcPacket) -> Self {
		Self {
			sequence: ibc_packet.sequence.into(),
			source_port: ibc_packet.source_port.into(),
			source_channel: ibc_packet.source_channel.into(),
			destination_port: ibc_packet.destination_port.into(),
			destination_channel: ibc_packet.destination_channel.into(),
			data: ibc_packet.data,
			timeout_height: match ibc_packet.timeout_height {
				IbcTimeoutHeight::Never => TimeoutHeight::Never,
				IbcTimeoutHeight::At(value) =>
					TimeoutHeight::At(Height::new(value.revision_number(), value.revision_height())),
			},
			timeout_timestamp: ibc_packet.timeout_timestamp.into(),
		}
	}
}

impl From<Packet> for IbcPacket {
	fn from(packet: Packet) -> Self {
		Self {
			sequence: packet.sequence.into(),
			source_port: packet.source_port.into(),
			source_channel: packet.source_channel.into(),
			destination_port: packet.destination_port.into(),
			destination_channel: packet.destination_channel.into(),
			data: packet.data,
			timeout_height: match packet.timeout_height {
				TimeoutHeight::Never => IbcTimeoutHeight::Never,
				TimeoutHeight::At(value) => IbcTimeoutHeight::At(
					IbcHeight::new(value.revision_number, value.revision_height).unwrap(),
				),
			},
			timeout_timestamp: packet.timeout_timestamp.into(),
		}
	}
}
