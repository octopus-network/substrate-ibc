use crate::ibc_core::ics24_host::{ChannelId, Height, PortId, Sequence, Timestamp};
use codec::{Decode, Encode};
use ibc::core::ics04_channel::packet::Packet as IbcPacket;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

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
	fn from(ibc_packet: IbcPacket) -> Self {
		Self {
			sequence: ibc_packet.sequence.into(),
			source_port: ibc_packet.source_port.into(),
			source_channel: ibc_packet.source_channel.into(),
			destination_port: ibc_packet.destination_port.into(),
			destination_channel: ibc_packet.destination_channel.into(),
			data: ibc_packet.data,
			timeout_height: ibc_packet.timeout_height.into(),
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
			timeout_height: packet.timeout_height.into(),
			timeout_timestamp: packet.timeout_timestamp.into(),
		}
	}
}
