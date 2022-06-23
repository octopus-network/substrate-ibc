use scale_info::prelude::{
	string::{String, ToString},
	vec::Vec,
};
use sp_std::str::FromStr;

use codec::{Decode, Encode};
use ibc::{
	self,
	core::{
		ics02_client::{client_type::ClientType as IbcClientType, height::Height as IbcHeight},
		ics04_channel::packet::Sequence as IbcSequence,
		ics24_host::identifier::{
			ChannelId as IbcChannelId, ClientId as IbcClientId, ConnectionId as IbcConnectionId,
			PortId as IbcPortId,
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
