use crate::{
	module::core::ics24_host::{ChannelId, ClientId, ClientType, ConnectionId, Height},
	prelude::{format, String},
	Config, Event,
};
use codec::{Decode, Encode};
use ibc::{core::ics26_routing, events::IbcEvent as RawIbcEvent};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_std::{str::FromStr, vec::Vec};

/// ibc-rs' `ModuleEvent` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ModuleEvent {
	pub kind: Vec<u8>,
	pub module_name: ModuleId,
	pub attributes: Vec<ModuleEventAttribute>,
}

impl From<ibc::events::ModuleEvent> for ModuleEvent {
	fn from(module_event: ibc::events::ModuleEvent) -> Self {
		Self {
			kind: module_event.kind.as_bytes().to_vec(),
			module_name: module_event.module_name.into(),
			attributes: module_event.attributes.into_iter().map(|event| event.into()).collect(),
		}
	}
}

impl From<ModuleEvent> for ibc::events::ModuleEvent {
	fn from(module_event: ModuleEvent) -> Self {
		Self {
			kind: String::from_utf8(module_event.kind).expect("never failed"),
			module_name: module_event.module_name.into(),
			attributes: module_event.attributes.into_iter().map(|event| event.into()).collect(),
		}
	}
}

/// ibc-rs' `ModuleId` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ModuleId(pub Vec<u8>);

impl From<ics26_routing::context::ModuleId> for ModuleId {
	fn from(module_id: ics26_routing::context::ModuleId) -> Self {
		Self(format!("{}", module_id).as_bytes().to_vec())
	}
}

impl From<ModuleId> for ics26_routing::context::ModuleId {
	fn from(module_id: ModuleId) -> Self {
		ics26_routing::context::ModuleId::from_str(&String::from_utf8(module_id.0).unwrap())
			.expect("should never fiaild")
	}
}

/// ibc-rs' `ModuleEventAttribute` representation in substrate
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ModuleEventAttribute {
	pub key: Vec<u8>,
	pub value: Vec<u8>,
}

impl From<ibc::events::ModuleEventAttribute> for ModuleEventAttribute {
	fn from(module_event_attribute: ibc::events::ModuleEventAttribute) -> Self {
		Self {
			key: module_event_attribute.key.as_bytes().to_vec(),
			value: module_event_attribute.value.as_bytes().to_vec(),
		}
	}
}

impl From<ModuleEventAttribute> for ibc::events::ModuleEventAttribute {
	fn from(module_event_attribute: ModuleEventAttribute) -> Self {
		Self {
			key: String::from_utf8(module_event_attribute.key).expect("should not be filled"),
			value: String::from_utf8(module_event_attribute.value).expect("should not be filled"),
		}
	}
}

impl<T: Config> From<RawIbcEvent> for Event<T> {
	fn from(raw_ibc_event: RawIbcEvent) -> Self {
		match raw_ibc_event {
			RawIbcEvent::CreateClient(create_client) => {
				let client_id = create_client.client_id();
				let client_type = create_client.client_type();
				let consensus_height = create_client.consensus_height();
				Event::<T>::CreateClient {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
					consensus_height: Height::from(consensus_height.clone()),
				}
			},
			RawIbcEvent::UpdateClient(update_client) => {
				let client_id = update_client.client_id();
				let client_type = update_client.client_type();
				let consensus_height = update_client.consensus_height();
				let consensus_heights = update_client.consensus_heights();
				let header = update_client.header();
				Event::<T>::UpdateClient {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
					consensus_height: Height::from(consensus_height.clone()),
					consensus_heights: consensus_heights
						.into_iter()
						.map(|value| Height::from(*value))
						.collect(),
					header: header.clone().into(),
				}
			},
			// Upgrade client events are not currently being used
			RawIbcEvent::UpgradeClient(upgrade_client) => {
				let client_id = upgrade_client.client_id();
				let client_type = upgrade_client.client_type();
				let consensus_height = upgrade_client.consensus_height();
				Event::<T>::UpgradeClient {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
					consensus_height: Height::from(consensus_height.clone()),
				}
			},
			RawIbcEvent::ClientMisbehaviour(client_misbehaviour) => {
				let client_id = client_misbehaviour.client_id();
				let client_type = client_misbehaviour.client_type();

				Event::<T>::ClientMisbehaviour {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
				}
			},
			RawIbcEvent::OpenInitConnection(open_init_connection) => {
				let connection_id =
					open_init_connection.0.connection_id.clone().map(|val| val.into());
				let client_id = open_init_connection.0.client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> = open_init_connection
					.0
					.counterparty_connection_id
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id = open_init_connection.0.counterparty_client_id.clone();

				Event::<T>::OpenInitConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenTryConnection(open_try_connection) => {
				let connection_id =
					open_try_connection.0.connection_id.clone().map(|val| val.into());
				let client_id = open_try_connection.0.client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> = open_try_connection
					.0
					.counterparty_connection_id
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id = open_try_connection.0.counterparty_client_id.clone();

				Event::<T>::OpenTryConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenAckConnection(open_ack_connection) => {
				let connection_id =
					open_ack_connection.0.connection_id.clone().map(|val| val.into());
				let client_id = open_ack_connection.0.client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> = open_ack_connection
					.0
					.counterparty_connection_id
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id = open_ack_connection.0.counterparty_client_id.clone();

				Event::<T>::OpenAckConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenConfirmConnection(open_confirm_connection) => {
				let connection_id =
					open_confirm_connection.0.connection_id.clone().map(|val| val.into());
				let client_id = open_confirm_connection.0.client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> = open_confirm_connection
					.0
					.counterparty_connection_id
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id =
					open_confirm_connection.0.counterparty_client_id.clone();

				Event::<T>::OpenConfirmConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenInitChannel(open_init_channel) => {
				let port_id = open_init_channel.port_id.clone();
				let channel_id: Option<ChannelId> =
					open_init_channel.channel_id.clone().map(|val| val.into());
				let connection_id = open_init_channel.connection_id.clone();
				let counterparty_port_id = open_init_channel.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					open_init_channel.channel_id.map(|val| val.into());

				Event::<T>::OpenInitChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenTryChannel(open_try_channel) => {
				let port_id = open_try_channel.port_id.clone();
				let channel_id: Option<ChannelId> =
					open_try_channel.channel_id.clone().map(|val| val.into());
				let connection_id = open_try_channel.connection_id.clone();
				let counterparty_port_id = open_try_channel.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					open_try_channel.channel_id.map(|val| val.into());

				Event::<T>::OpenTryChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenAckChannel(open_ack_channel) => {
				let port_id = open_ack_channel.port_id.clone();
				let channel_id: Option<ChannelId> =
					open_ack_channel.channel_id.clone().map(|val| val.into());
				let connection_id = open_ack_channel.connection_id.clone();
				let counterparty_port_id = open_ack_channel.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					open_ack_channel.channel_id.map(|val| val.into());

				Event::<T>::OpenAckChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenConfirmChannel(open_confirm_channel) => {
				let port_id = open_confirm_channel.port_id.clone();
				let channel_id: Option<ChannelId> =
					open_confirm_channel.channel_id.clone().map(|val| val.into());
				let connection_id = open_confirm_channel.connection_id.clone();
				let counterparty_port_id = open_confirm_channel.counterparty_port_id;
				let counterparty_channel_id: Option<ChannelId> =
					open_confirm_channel.channel_id.map(|val| val.into());

				Event::<T>::OpenConfirmChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::CloseInitChannel(close_init_channel) => {
				let port_id = close_init_channel.port_id.clone();
				let channel_id: Option<ChannelId> = Some(close_init_channel.channel_id.into());
				let connection_id = close_init_channel.connection_id.clone();
				let counterparty_port_id = close_init_channel.counterparty_port_id;
				let counterparty_channel_id: Option<ChannelId> =
					close_init_channel.counterparty_channel_id.map(|val| val.into());

				Event::<T>::CloseInitChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::CloseConfirmChannel(close_confirm_channel) => {
				let port_id = close_confirm_channel.port_id.clone();
				let channel_id: Option<ChannelId> =
					close_confirm_channel.channel_id.clone().map(|val| val.into());
				let connection_id = close_confirm_channel.connection_id.clone();
				let counterparty_port_id = close_confirm_channel.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					close_confirm_channel.channel_id.map(|val| val.into());

				Event::<T>::CloseConfirmChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::SendPacket(send_packet) => {
				let packet = send_packet.packet;

				Event::<T>::SendPacket { packet: packet.into() }
			},
			RawIbcEvent::ReceivePacket(receiver_packet) => {
				let packet = receiver_packet.packet;

				Event::<T>::ReceivePacket { packet: packet.into() }
			},
			RawIbcEvent::WriteAcknowledgement(write_acknowledgement) => {
				let packet = write_acknowledgement.packet;
				let ack = write_acknowledgement.ack;

				Event::<T>::WriteAcknowledgement { packet: packet.into(), ack }
			},
			RawIbcEvent::AcknowledgePacket(acknowledge_packet) => {
				let packet = acknowledge_packet.packet;

				Event::<T>::AcknowledgePacket { packet: packet.into() }
			},
			RawIbcEvent::TimeoutPacket(time_out_packet) => {
				let packet = time_out_packet.packet;

				Event::<T>::TimeoutPacket { packet: packet.into() }
			},
			RawIbcEvent::TimeoutOnClosePacket(timeout_on_close_packet) => {
				let packet = timeout_on_close_packet.packet;

				Event::<T>::TimeoutOnClosePacket { packet: packet.into() }
			},
			RawIbcEvent::AppModule(app_module) => Event::<T>::AppModule(app_module.into()),
		}
	}
}
