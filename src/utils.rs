use crate::Config;
use codec::Encode;
use scale_info::prelude::{fmt::Debug, format, vec::Vec};

use super::*;
use ibc::{
	applications::transfer::{error::Error as Ics20Error, VERSION},
	core::ics24_host::identifier::{ChannelId as IbcChannelId, PortId},
	signer::Signer,
};

use ibc::{
	core::{
		ics02_client::msgs::ClientMsg,
		ics03_connection::msgs::ConnectionMsg,
		ics04_channel::msgs::{ChannelMsg, PacketMsg},
		ics26_routing::{handler, msgs::Ics26Envelope},
	},
	events::IbcEvent,
};

pub fn host_height<T: Config>() -> u64 {
	let block_number = format!("{:?}", <frame_system::Pallet<T>>::block_number());
	let current_height: u64 = block_number.parse().unwrap_or_default();
	current_height
}

pub fn get_channel_escrow_address(
	port_id: &PortId,
	channel_id: &IbcChannelId,
) -> Result<Signer, Ics20Error> {
	let contents = format!("{}/{}", port_id, channel_id);
	let mut data = VERSION.as_bytes().to_vec();
	data.extend_from_slice(&[0]);
	data.extend_from_slice(contents.as_bytes());

	let hash = sp_io::hashing::sha2_256(&data).to_vec();
	let mut hex_string = hex::encode_upper(hash);
	hex_string.insert_str(0, "0x");
	hex_string.parse::<Signer>().map_err(Ics20Error::signer)
}

pub fn event_from_ibc_event<T: Config>(value: IbcEvent) -> Event<T> {
	match value {
		IbcEvent::NewBlock(value) => Event::NewBlock(value.height.into()),
		IbcEvent::CreateClient(value) => {
			let height = value.0.height;
			let client_id = value.0.client_id;
			let client_type = value.0.client_type;
			let consensus_height = value.0.consensus_height;
			Event::CreateClient(
				height.into(),
				client_id.into(),
				client_type.into(),
				consensus_height.into(),
			)
		},
		IbcEvent::UpdateClient(value) => {
			let height = value.common.height;
			let client_id = value.common.client_id;
			let client_type = value.common.client_type;
			let consensus_height = value.common.consensus_height;
			Event::UpdateClient(
				height.into(),
				client_id.into(),
				client_type.into(),
				consensus_height.into(),
			)
		},
		// Upgrade client events are not currently being used
		IbcEvent::UpgradeClient(value) => {
			let height = value.0.height;
			let client_id = value.0.client_id;
			let client_type = value.0.client_type;
			let consensus_height = value.0.consensus_height;
			Event::UpgradeClient(
				height.into(),
				client_id.into(),
				client_type.into(),
				consensus_height.into(),
			)
		},
		IbcEvent::ClientMisbehaviour(value) => {
			let height = value.0.height;
			let client_id = value.0.client_id;
			let client_type = value.0.client_type;
			let consensus_height = value.0.consensus_height;
			Event::ClientMisbehaviour(
				height.into(),
				client_id.into(),
				client_type.into(),
				consensus_height.into(),
			)
		},
		IbcEvent::OpenInitConnection(value) => {
			let height = value.attributes().height;
			let connection_id: Option<ConnectionId> =
				value.attributes().connection_id.clone().map(|val| val.into());
			let client_id = value.attributes().client_id.clone();
			let counterparty_connection_id: Option<ConnectionId> =
				value.attributes().counterparty_connection_id.clone().map(|val| val.into());

			let counterparty_client_id = value.attributes().counterparty_client_id.clone();
			Event::OpenInitConnection(
				height.into(),
				connection_id,
				client_id.into(),
				counterparty_connection_id,
				counterparty_client_id.into(),
			)
		},
		IbcEvent::OpenTryConnection(value) => {
			let height = value.attributes().height;
			let connection_id: Option<ConnectionId> =
				value.attributes().connection_id.clone().map(|val| val.into());
			let client_id = value.attributes().client_id.clone();
			let counterparty_connection_id: Option<ConnectionId> =
				value.attributes().counterparty_connection_id.clone().map(|val| val.into());

			let counterparty_client_id = value.attributes().counterparty_client_id.clone();
			Event::OpenTryConnection(
				height.into(),
				connection_id,
				client_id.into(),
				counterparty_connection_id,
				counterparty_client_id.into(),
			)
		},
		IbcEvent::OpenAckConnection(value) => {
			let height = value.attributes().height;
			let connection_id: Option<ConnectionId> =
				value.attributes().connection_id.clone().map(|val| val.into());
			let client_id = value.attributes().client_id.clone();
			let counterparty_connection_id: Option<ConnectionId> =
				value.attributes().counterparty_connection_id.clone().map(|val| val.into());

			let counterparty_client_id = value.attributes().counterparty_client_id.clone();
			Event::OpenAckConnection(
				height.into(),
				connection_id,
				client_id.into(),
				counterparty_connection_id,
				counterparty_client_id.into(),
			)
		},
		IbcEvent::OpenConfirmConnection(value) => {
			let height = value.attributes().height;
			let connection_id: Option<ConnectionId> =
				value.attributes().connection_id.clone().map(|val| val.into());
			let client_id = value.attributes().client_id.clone();
			let counterparty_connection_id: Option<ConnectionId> =
				value.attributes().counterparty_connection_id.clone().map(|val| val.into());

			let counterparty_client_id = value.attributes().counterparty_client_id.clone();
			Event::OpenConfirmConnection(
				height.into(),
				connection_id,
				client_id.into(),
				counterparty_connection_id,
				counterparty_client_id.into(),
			)
		},
		IbcEvent::OpenInitChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
			let connection_id = value.connection_id.clone();
			let counterparty_port_id = value.counterparty_port_id.clone();
			let counterparty_channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
			Event::OpenInitChannel(
				height.into(),
				port_id.into(),
				channel_id,
				connection_id.into(),
				counterparty_port_id.into(),
				counterparty_channel_id,
			)
		},
		IbcEvent::OpenTryChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
			let connection_id = value.connection_id.clone();
			let counterparty_port_id = value.counterparty_port_id.clone();
			let counterparty_channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
			Event::OpenTryChannel(
				height.into(),
				port_id.into(),
				channel_id,
				connection_id.into(),
				counterparty_port_id.into(),
				counterparty_channel_id,
			)
		},
		IbcEvent::OpenAckChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
			let connection_id = value.connection_id.clone();
			let counterparty_port_id = value.counterparty_port_id.clone();
			let counterparty_channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
			Event::OpenAckChannel(
				height.into(),
				port_id.into(),
				channel_id,
				connection_id.into(),
				counterparty_port_id.into(),
				counterparty_channel_id,
			)
		},
		IbcEvent::OpenConfirmChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
			let connection_id = value.connection_id.clone();
			let counterparty_port_id = value.counterparty_port_id;
			let counterparty_channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
			Event::OpenConfirmChannel(
				height.into(),
				port_id.into(),
				channel_id,
				connection_id.into(),
				counterparty_port_id.into(),
				counterparty_channel_id,
			)
		},
		IbcEvent::CloseInitChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = Some(value.channel_id.into());
			let connection_id = value.connection_id.clone();
			let counterparty_port_id = value.counterparty_port_id;
			let counterparty_channel_id: Option<ChannelId> =
				value.counterparty_channel_id.map(|val| val.into());
			Event::CloseInitChannel(
				height.into(),
				port_id.into(),
				channel_id,
				connection_id.into(),
				counterparty_port_id.into(),
				counterparty_channel_id,
			)
		},
		IbcEvent::CloseConfirmChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
			let connection_id = value.connection_id.clone();
			let counterparty_port_id = value.counterparty_port_id.clone();
			let counterparty_channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
			Event::CloseConfirmChannel(
				height.into(),
				port_id.into(),
				channel_id,
				connection_id.into(),
				counterparty_port_id.into(),
				counterparty_channel_id,
			)
		},
		IbcEvent::SendPacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::SendPacket(height.into(), packet.into())
		},
		IbcEvent::ReceivePacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::ReceivePacket(height.into(), packet.into())
		},
		IbcEvent::WriteAcknowledgement(value) => {
			let height = value.height;
			let packet = value.packet;
			let ack = value.ack;
			Event::WriteAcknowledgement(height.into(), packet.into(), ack)
		},
		IbcEvent::AcknowledgePacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::AcknowledgePacket(height.into(), packet.into())
		},
		IbcEvent::TimeoutPacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::TimeoutPacket(height.into(), packet.into())
		},
		IbcEvent::TimeoutOnClosePacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::TimeoutOnClosePacket(height.into(), packet.into())
		},
		IbcEvent::ChainError(value) => Event::ChainError(value.as_bytes().to_vec()),
		// TODO(davirain)
		IbcEvent::AppModule(value) => unimplemented!(),
	}
}
