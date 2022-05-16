use crate::*;
use ibc::{
	core::{
		ics02_client::msgs::ClientMsg,
		ics03_connection::msgs::ConnectionMsg,
		ics04_channel::msgs::{ChannelMsg, PacketMsg},
		ics26_routing::{handler, msgs::Ics26Envelope},
	},
	events::IbcEvent,
	signer::Signer,
};

pub fn get_signer<T: Config>(
	message: ibc_proto::google::protobuf::Any,
) -> Result<Signer, DispatchError> {
	let decode_message = handler::decode(message).map_err(|_| Error::<T>::InvalidDecode)?;
	let signer = match decode_message {
		Ics26Envelope::Ics2Msg(value) => match value {
			ClientMsg::CreateClient(val) => val.signer,
			ClientMsg::UpdateClient(val) => val.signer,
			ClientMsg::Misbehaviour(val) => val.signer,
			ClientMsg::UpgradeClient(val) => val.signer,
		},
		Ics26Envelope::Ics3Msg(value) => match value {
			ConnectionMsg::ConnectionOpenInit(val) => val.signer,
			ConnectionMsg::ConnectionOpenTry(val) => val.signer,
			ConnectionMsg::ConnectionOpenAck(val) => val.signer,
			ConnectionMsg::ConnectionOpenConfirm(val) => val.signer,
		},
		Ics26Envelope::Ics4ChannelMsg(value) => match value {
			ChannelMsg::ChannelOpenInit(val) => val.signer,
			ChannelMsg::ChannelOpenTry(val) => val.signer,
			ChannelMsg::ChannelOpenAck(val) => val.signer,
			ChannelMsg::ChannelOpenConfirm(val) => val.signer,
			ChannelMsg::ChannelCloseInit(val) => val.signer,
			ChannelMsg::ChannelCloseConfirm(val) => val.signer,
		},
		Ics26Envelope::Ics4PacketMsg(value) => match value {
			PacketMsg::RecvPacket(val) => val.signer,
			PacketMsg::AckPacket(val) => val.signer,
			PacketMsg::ToPacket(val) => val.signer,
			PacketMsg::ToClosePacket(val) => val.signer,
		},
		Ics26Envelope::Ics20Msg(value) => value.sender,
	};

	Ok(signer)
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
		// UpgradeClient(
		// 	height: Height,
		// 	client_id: ClientId,
		// 	client_type: ClientType,
		// 	consensus_height: Height,
		// )
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
		ibc::events::IbcEvent::OpenConfirmConnection(value) => {
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
			let channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
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
		ibc::events::IbcEvent::OpenTryChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
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
			let channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
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
			let channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
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
		ibc::events::IbcEvent::CloseInitChannel(value) => {
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
			let channel_id: Option<ChannelId> = value.channel_id.map(|val| val.into());
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
		IbcEvent::Empty(value) => Event::Empty(value.as_bytes().to_vec()),
		IbcEvent::ChainError(value) => Event::ChainError(value.as_bytes().to_vec()),
	}
}
