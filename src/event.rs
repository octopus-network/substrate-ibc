pub mod primitive {
	use crate::alloc::string::ToString;
	use alloc::string::String;
	use ibc::{
		ics02_client::{client_type::ClientType as IbcClientType, height::Height as IbcHeight},
		ics04_channel::packet::{Packet as IbcPacket, Sequence as IbcSequence},
		ics24_host::identifier::{
			ChannelId as IbcChannelId, ClientId as IbcClientId, ConnectionId as IbcConnectionId,
			PortId as IbcPortId,
		},
		timestamp::Timestamp as IbcTimestamp,
	};
	use sp_std::{str::FromStr, vec::Vec};

	use codec::{Decode, Encode};
	use scale_info::{
		build::*,
		MetaType,
		Path,
		Type,
		TypeInfo,
		prelude::vec,
	};
	use sp_runtime::RuntimeDebug;

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct PortId(pub Vec<u8>);


	impl TypeInfo for PortId {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("PortId", module_path!()))
				.composite(Fields::unnamed()
					.field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")))
		}
	}

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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct ChannelId(pub Vec<u8>);

	impl TypeInfo for ChannelId {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("ChannelId", module_path!()))
				.composite(Fields::unnamed()
					.field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")))
		}
	}


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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct Height {
		/// Previously known as "epoch"
		pub revision_number: u64,

		/// The height of a block
		pub revision_height: u64,
	}

	impl TypeInfo for Height {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("Height", module_path!()))
				.composite(Fields::named()
					.field(|f| f.ty::<u64>().name("revision_number").type_name("u64"))
					.field(|f| f.ty::<u64>().name("revision_height").type_name("u64")))
		}
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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub enum ClientType {
		Tendermint,
		Grandpa,
	}

	impl TypeInfo for ClientType {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("ClientTy", module_path!()))
				.variant(
					Variants::new()
						.variant("Tendermint", |v| v.index(1))
						.variant("Grandpa", |v| v.index(2))
				)
		}
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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct ClientId(pub Vec<u8>);

	impl TypeInfo for ClientId {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("ClientId", module_path!()))
				.composite(Fields::unnamed()
					.field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")))
		}
	}

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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct ConnectionId(pub Vec<u8>);

	impl TypeInfo for ConnectionId {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("ConnectionId", module_path!()))
				.composite(Fields::unnamed()
					.field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")))
		}
	}


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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct Timestamp {
		pub time: Vec<u8>,
	}

	impl TypeInfo for Timestamp {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("Timestamp", module_path!()))
				.composite(Fields::unnamed()
					.field(|f| f.ty::<Vec<u8>>().type_name("Vec<u8>")))
		}
	}

	impl From<IbcTimestamp> for Timestamp {
		fn from(val: IbcTimestamp) -> Self {
			Self { time: val.as_nanoseconds().to_string().as_bytes().to_vec() }
		}
	}

	impl Timestamp {
		pub fn to_ibc_timestamp(self) -> IbcTimestamp {
			let value = String::from_utf8(self.time).unwrap();
			let timestamp = IbcTimestamp::from_str(&value).unwrap();
			timestamp
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct Sequence(u64);

	impl TypeInfo for Sequence {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("Sequence", module_path!()))
				.composite(Fields::unnamed()
					.field(|f| f.ty::<u64>().type_name("u64")))
		}
	}

	impl From<IbcSequence> for Sequence {
		fn from(val: IbcSequence) -> Self {
			Self(val.0)
		}
	}

	impl Sequence {
		pub fn to_ibc_sequence(self) -> IbcSequence {
			IbcSequence(self.0)
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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


	impl TypeInfo for Packet {
		type Identity = Self;

		fn type_info() -> Type {
			Type::builder()
				.path(Path::new("Packet", module_path!()))
				.composite(Fields::named()
					.field(|f| f.ty::<Sequence>().name("sequence").type_name("Sequence"))
					.field(|f| f.ty::<PortId>().name("source_port").type_name("PortId"))
					.field(|f| f.ty::<ChannelId>().name("source_channel").type_name("ChannelId"))
					.field(|f| f.ty::<PortId>().name("destination_port").type_name("PortId"))
					.field(|f| f.ty::<ChannelId>().name("destination_channel").type_name("ChannelId"))
					.field(|f| f.ty::<Vec<u8>>().name("data").type_name("Vec<u8>"))
					.field(|f| f.ty::<Height>().name("timeout_height").type_name("Height"))
					.field(|f| f.ty::<Timestamp>().name("timeout_timestamp").type_name("Timestamp"))
				)

		}
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
}
