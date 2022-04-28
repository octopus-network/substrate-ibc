use crate::*;

pub fn get_signer<T: Config>(
	message: ibc_proto::google::protobuf::Any,
) -> Result<ibc::signer::Signer, DispatchError> {
	let decode_message = ibc::core::ics26_routing::handler::decode(message)
		.map_err(|_| Error::<T>::InvalidDecode)?;
	let signer = match decode_message {
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics2Msg(value) => match value {
			ibc::core::ics02_client::msgs::ClientMsg::CreateClient(val) => val.signer,
			ibc::core::ics02_client::msgs::ClientMsg::UpdateClient(val) => val.signer,
			ibc::core::ics02_client::msgs::ClientMsg::Misbehaviour(val) => val.signer,
			ibc::core::ics02_client::msgs::ClientMsg::UpgradeClient(val) => val.signer,
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics3Msg(value) => match value {
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenInit(val) => val.signer,
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenTry(val) =>
				val.signer.clone(),
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenAck(val) =>
				val.signer.clone(),
			ibc::core::ics03_connection::msgs::ConnectionMsg::ConnectionOpenConfirm(val) =>
				val.signer,
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics4ChannelMsg(value) => match value {
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenInit(val) => val.signer,
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenTry(val) => val.signer,
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenAck(val) => val.signer,
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelOpenConfirm(val) => val.signer,
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelCloseInit(val) => val.signer,
			ibc::core::ics04_channel::msgs::ChannelMsg::ChannelCloseConfirm(val) => val.signer,
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics4PacketMsg(value) => match value {
			ibc::core::ics04_channel::msgs::PacketMsg::RecvPacket(val) => val.signer,
			ibc::core::ics04_channel::msgs::PacketMsg::AckPacket(val) => val.signer,
			ibc::core::ics04_channel::msgs::PacketMsg::ToPacket(val) => val.signer,
			ibc::core::ics04_channel::msgs::PacketMsg::ToClosePacket(val) => val.signer,
		},
		ibc::core::ics26_routing::msgs::Ics26Envelope::Ics20Msg(value) => value.sender,
	};

	Ok(signer)
}

pub fn event_from_ibc_event<T: Config>(value: ibc::events::IbcEvent) -> Event<T> {
	match value {
		ibc::events::IbcEvent::NewBlock(value) => Event::NewBlock(value.height.into()),
		ibc::events::IbcEvent::CreateClient(value) => {
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
		ibc::events::IbcEvent::UpdateClient(value) => {
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
		ibc::events::IbcEvent::UpgradeClient(value) => {
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
		ibc::events::IbcEvent::ClientMisbehaviour(value) => {
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
		ibc::events::IbcEvent::OpenInitConnection(value) => {
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
		ibc::events::IbcEvent::OpenTryConnection(value) => {
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
		ibc::events::IbcEvent::OpenAckConnection(value) => {
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
		ibc::events::IbcEvent::OpenInitChannel(value) => {
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
		ibc::events::IbcEvent::OpenTryChannel(value) => {
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
		ibc::events::IbcEvent::OpenAckChannel(value) => {
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
		ibc::events::IbcEvent::OpenConfirmChannel(value) => {
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
		ibc::events::IbcEvent::CloseInitChannel(value) => {
			let height = value.height;
			let port_id = value.port_id.clone();
			let channel_id: Option<ChannelId> = Some(value.channel_id.clone().into());
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
		ibc::events::IbcEvent::CloseConfirmChannel(value) => {
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
		ibc::events::IbcEvent::SendPacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::SendPacket(height.into(), packet.into())
		},
		ibc::events::IbcEvent::ReceivePacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::ReceivePacket(height.into(), packet.into())
		},
		ibc::events::IbcEvent::WriteAcknowledgement(value) => {
			let height = value.height;
			let packet = value.packet;
			let ack = value.ack;
			Event::WriteAcknowledgement(height.into(), packet.into(), ack)
		},
		ibc::events::IbcEvent::AcknowledgePacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::AcknowledgePacket(height.into(), packet.into())
		},
		ibc::events::IbcEvent::TimeoutPacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::TimeoutPacket(height.into(), packet.into())
		},
		ibc::events::IbcEvent::TimeoutOnClosePacket(value) => {
			let height = value.height;
			let packet = value.packet;
			Event::TimeoutOnClosePacket(height.into(), packet.into())
		},
		ibc::events::IbcEvent::Empty(value) => Event::Empty(value.as_bytes().to_vec()),
		ibc::events::IbcEvent::ChainError(value) => Event::ChainError(value.as_bytes().to_vec()),
		_ => unimplemented!(),
	}
}
