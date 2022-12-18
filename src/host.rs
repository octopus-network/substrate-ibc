use core::marker::PhantomData;

use crate::{Config, Error, REVISION_NUMBER};
pub use alloc::{
	format,
	string::{String, ToString},
};
use ibc::{
	core::{
		ics02_client::{
			client_type::ClientType as IbcClientType, error::ClientError,
			height::Height as IbcHeight,
		},
		ics04_channel::{
			channel::Order as IbcOrder,
			packet::{Packet as IbcPacket, Sequence as IbcSequence},
			Version as IbcVersion,
		},
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

pub const TENDERMINT_CLIENT_TYPE: &'static str = "07-tendermint";
#[cfg(test)]
pub const MOCK_CLIENT_TYPE: &'static str = "9999-mock";

/// ibc-rs' `PortId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PortId<T> {
	pub raw: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T: Config> From<IbcPortId> for PortId<T> {
	fn from(ibc_port_id: IbcPortId) -> Self {
		let value = ibc_port_id.as_str().as_bytes().to_vec();
		Self { raw: value, phantom: PhantomData::default() }
	}
}

impl<T: Config> TryFrom<PortId<T>> for IbcPortId {
	type Error = Error<T>;
	fn try_from(port_id: PortId<T>) -> Result<Self, Self::Error> {
		IbcPortId::from_str(
			&String::from_utf8(port_id.raw).map_err(|_| Error::<T>::DecodeStringFailed)?,
		)
		.map_err(|_| Error::<T>::InvalidPortId)
	}
}

/// ibc-rs' `ChannelId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ChannelId<T> {
	pub raw: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T> From<IbcChannelId> for ChannelId<T> {
	fn from(ibc_channel_id: IbcChannelId) -> Self {
		let value = ibc_channel_id.to_string().as_bytes().to_vec();
		Self { raw: value, phantom: PhantomData::default() }
	}
}

impl<T: Config> TryFrom<ChannelId<T>> for IbcChannelId {
	type Error = Error<T>;

	fn try_from(channel_id: ChannelId<T>) -> Result<Self, Self::Error> {
		let value =
			String::from_utf8(channel_id.raw).map_err(|_| Error::<T>::DecodeStringFailed)?;
		Self::from_str(&value).map_err(|_| Error::<T>::InvalidChannelId)
	}
}

/// ibc-rs' `TimeoutHeight` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum TimeoutHeight<T> {
	Never,
	At(Height<T>),
}

impl<T> From<IbcTimeoutHeight> for TimeoutHeight<T> {
	fn from(ibc_time_height: IbcTimeoutHeight) -> Self {
		match ibc_time_height {
			IbcTimeoutHeight::Never => Self::Never,
			IbcTimeoutHeight::At(height) => Self::At(height.into()),
		}
	}
}

impl<T: Config> TryFrom<TimeoutHeight<T>> for IbcTimeoutHeight {
	type Error = Error<T>;
	fn try_from(time_height: TimeoutHeight<T>) -> Result<Self, Self::Error> {
		match time_height {
			TimeoutHeight::Never => Ok(Self::Never),
			TimeoutHeight::At(height) => Ok(Self::At(height.try_into()?)),
		}
	}
}

/// ibc-rs' `Height` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Height<T> {
	/// Previously known as "epoch"
	pub revision_number: u64,

	/// The height of a block
	pub revision_height: u64,

	phantom: PhantomData<T>,
}

impl<T> From<IbcHeight> for Height<T> {
	fn from(ibc_height: IbcHeight) -> Self {
		Height::new(ibc_height.revision_number(), ibc_height.revision_height())
	}
}

impl<T: Config> TryFrom<Height<T>> for IbcHeight {
	type Error = Error<T>;
	fn try_from(height: Height<T>) -> Result<Self, Self::Error> {
		IbcHeight::new(REVISION_NUMBER, height.revision_height)
			.map_err(|_| Error::<T>::InvalidHeight)
	}
}

impl<T> Height<T> {
	pub fn new(revision_number: u64, revision_height: u64) -> Self {
		Self { revision_number, revision_height, phantom: PhantomData::default() }
	}
}

/// ibc-rs' `ClientType` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientType<T> {
	pub raw: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T: Config> ClientType<T> {
	pub fn new(s: &str) -> Self {
		let value = s.as_bytes().to_vec();
		Self { raw: value, phantom: PhantomData::default() }
	}

	pub fn to_string(&self) -> Result<String, Error<T>> {
		String::from_utf8(self.raw.clone()).map_err(|_| Error::<T>::DecodeStringFailed)
	}
}

impl<T: Config> From<IbcClientType> for ClientType<T> {
	fn from(ibc_client_type: IbcClientType) -> Self {
		Self::new(ibc_client_type.as_str())
	}
}

impl<T: Config> TryFrom<ClientType<T>> for IbcClientType {
	type Error = Error<T>;

	fn try_from(client_type: ClientType<T>) -> Result<Self, Self::Error> {
		match client_type.to_string()?.as_str() {
			"07-tendermint" => Ok(IbcClientType::new(TENDERMINT_CLIENT_TYPE.into())),
			#[cfg(test)]
			"9999-mock" => Ok(IbcClientType::new(MOCK_CLIENT_TYPE.into())),
			_ => Err(Error::<T>::UnknownClientType),
		}
	}
}

/// ibc-rs' `ClientId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ClientId<T> {
	pub raw: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T> From<IbcClientId> for ClientId<T> {
	fn from(ibc_client_id: IbcClientId) -> Self {
		let value = ibc_client_id.as_str().as_bytes().to_vec();
		Self { raw: value, phantom: PhantomData::default() }
	}
}

impl<T: Config> TryFrom<ClientId<T>> for IbcClientId {
	type Error = Error<T>;

	fn try_from(client_id: ClientId<T>) -> Result<Self, Self::Error> {
		let value = String::from_utf8(client_id.raw).map_err(|_| Error::<T>::DecodeStringFailed)?;
		IbcClientId::from_str(&value).map_err(|_| Error::<T>::InvalidClientId)
	}
}

/// ibc-rs' `ConnectionId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ConnectionId<T> {
	pub raw: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T> From<IbcConnectionId> for ConnectionId<T> {
	fn from(ibc_connection_id: IbcConnectionId) -> Self {
		let value = ibc_connection_id.as_str().as_bytes().to_vec();
		Self { raw: value, phantom: PhantomData::default() }
	}
}

impl<T: Config> TryFrom<ConnectionId<T>> for IbcConnectionId {
	type Error = Error<T>;

	fn try_from(connection_id: ConnectionId<T>) -> Result<Self, Self::Error> {
		let value =
			String::from_utf8(connection_id.raw).map_err(|_| Error::<T>::DecodeStringFailed)?;
		IbcConnectionId::from_str(&value).map_err(|_| Error::<T>::InvalidConnectionId)
	}
}

/// ibc-rs' `Timestamp` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Timestamp<T> {
	pub time: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T> From<IbcTimestamp> for Timestamp<T> {
	fn from(ibc_timestamp: IbcTimestamp) -> Self {
		Self {
			time: ibc_timestamp.nanoseconds().to_string().as_bytes().to_vec(),
			phantom: PhantomData::default(),
		}
	}
}

impl<T: Config> TryFrom<Timestamp<T>> for IbcTimestamp {
	type Error = Error<T>;

	fn try_from(timestamp: Timestamp<T>) -> Result<Self, Self::Error> {
		let value =
			String::from_utf8(timestamp.time).map_err(|_| Error::<T>::DecodeStringFailed)?;
		Self::from_str(&value).map_err(|_| Error::<T>::InvalidTimestamp)
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
pub struct Packet<T> {
	pub sequence: Sequence,
	pub source_port: PortId<T>,
	pub source_channel: ChannelId<T>,
	pub destination_port: PortId<T>,
	pub destination_channel: ChannelId<T>,
	pub data: Vec<u8>,
	pub timeout_height: TimeoutHeight<T>,
	pub timeout_timestamp: Timestamp<T>,
}

impl<T: Config> From<IbcPacket> for Packet<T> {
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

impl<T: Config> TryFrom<Packet<T>> for IbcPacket {
	type Error = Error<T>;

	fn try_from(packet: Packet<T>) -> Result<Self, Self::Error> {
		Ok(Self {
			sequence: packet.sequence.into(),
			source_port: packet.source_port.try_into()?,
			source_channel: packet.source_channel.try_into()?,
			destination_port: packet.destination_port.try_into()?,
			destination_channel: packet.destination_channel.try_into()?,
			data: packet.data,
			timeout_height: match packet.timeout_height {
				TimeoutHeight::Never => IbcTimeoutHeight::Never,
				TimeoutHeight::At(value) => IbcTimeoutHeight::At(
					IbcHeight::new(value.revision_number, value.revision_height)
						.map_err(|_| Error::<T>::InvalidHeight)?,
				),
			},
			timeout_timestamp: packet.timeout_timestamp.try_into()?,
		})
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Version<T> {
	pub raw: Vec<u8>,
	phantom: PhantomData<T>,
}

impl<T> From<IbcVersion> for Version<T> {
	fn from(ibc_versoion: IbcVersion) -> Self {
		let value = ibc_versoion.to_string().as_bytes().to_vec();
		Self { raw: value, phantom: PhantomData::default() }
	}
}

impl<T: Config> TryFrom<Version<T>> for IbcVersion {
	type Error = Error<T>;

	fn try_from(version: Version<T>) -> Result<Self, Self::Error> {
		let value = String::from_utf8(version.raw).map_err(|_| Error::<T>::DecodeStringFailed)?;
		IbcVersion::from_str(&value).map_err(|_| Error::<T>::InvalidVersion)
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum Order {
	None = 0,
	Unordered = 1,
	Ordered = 2,
}

impl From<IbcOrder> for Order {
	fn from(ibc_order: IbcOrder) -> Self {
		match ibc_order {
			IbcOrder::None => Self::None,
			IbcOrder::Unordered => Self::Unordered,
			IbcOrder::Ordered => Self::Ordered,
		}
	}
}

impl From<Order> for IbcOrder {
	fn from(order: Order) -> Self {
		match order {
			Order::None => Self::None,
			Order::Unordered => Self::Unordered,
			Order::Ordered => Self::Ordered,
		}
	}
}
