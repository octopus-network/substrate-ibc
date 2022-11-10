use crate::{
	module::core::ics24_host::{ClientId, ClientType, ConnectionId, Height},
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
					ConnectionId::from(open_init_connection.connection_id().clone());
				let client_id = open_init_connection.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = open_init_connection
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id = open_init_connection.counterparty_client_id().clone();

				Event::<T>::OpenInitConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenTryConnection(open_try_connection) => {
				let connection_id = ConnectionId::from(open_try_connection.connection_id().clone());
				let client_id = open_try_connection.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = open_try_connection
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id = open_try_connection.counterparty_client_id().clone();

				Event::<T>::OpenTryConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenAckConnection(open_ack_connection) => {
				let connection_id = ConnectionId::from(open_ack_connection.connection_id().clone());
				let client_id = open_ack_connection.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = open_ack_connection
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id = open_ack_connection.counterparty_client_id().clone();

				Event::<T>::OpenAckConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenConfirmConnection(open_confirm_connection) => {
				let connection_id =
					ConnectionId::from(open_confirm_connection.connection_id().clone());
				let client_id = open_confirm_connection.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = open_confirm_connection
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));
				let counterparty_client_id =
					open_confirm_connection.counterparty_client_id().clone();

				Event::<T>::OpenConfirmConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenInitChannel(open_init_channel) => {
				let port_id = open_init_channel.port_id().clone();
				let channel_id = open_init_channel.channel_id().clone();
				let counterparty_port_id = open_init_channel.counterparty_port_id().clone();
				let connection_id = open_init_channel.connection_id().clone();
				let version = open_init_channel.version().clone();

				Event::<T>::OpenInitChannel {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					connection_id: connection_id.into(),
					version: version.into(),
				}
			},
			RawIbcEvent::OpenTryChannel(open_try_channel) => {
				let port_id = open_try_channel.port_id().clone();
				let channel_id = open_try_channel.channel_id().clone();
				let counterparty_port_id = open_try_channel.counterparty_port_id().clone();
				let counterparty_channel_id = open_try_channel.counterparty_channel_id().clone();
				let connection_id = open_try_channel.connection_id().clone();
				let version = open_try_channel.version().clone();

				Event::<T>::OpenTryChannel {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id: counterparty_channel_id.into(),
					connection_id: connection_id.into(),
					version: version.into(),
				}
			},
			RawIbcEvent::OpenAckChannel(open_ack_channel) => {
				let port_id = open_ack_channel.port_id().clone();
				let channel_id = open_ack_channel.channel_id().clone();
				let counterparty_port_id = open_ack_channel.counterparty_port_id().clone();
				let counterparty_channel_id = open_ack_channel.counterparty_channel_id().clone();
				let connection_id = open_ack_channel.connection_id().clone();

				Event::<T>::OpenAckChannel {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id: counterparty_channel_id.into(),
					connection_id: connection_id.into(),
				}
			},
			RawIbcEvent::OpenConfirmChannel(open_confirm_channel) => {
				let port_id = open_confirm_channel.port_id().clone();
				let channel_id = open_confirm_channel.channel_id().clone();
				let counterparty_port_id = open_confirm_channel.counterparty_port_id().clone();
				let counterparty_channel_id =
					open_confirm_channel.counterparty_channel_id().clone();
				let connection_id = open_confirm_channel.connection_id().clone();

				Event::<T>::OpenConfirmChannel {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id: counterparty_channel_id.into(),
					connection_id: connection_id.into(),
				}
			},
			RawIbcEvent::CloseInitChannel(close_init_channel) => {
				let port_id = close_init_channel.port_id().clone();
				let channel_id = close_init_channel.channel_id().clone();
				let counterparty_port_id = close_init_channel.counterparty_port_id().clone();
				let counterparty_channel_id = close_init_channel.counterparty_channel_id().clone();
				let connection_id = close_init_channel.connection_id().clone();

				Event::<T>::CloseInitChannel {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id: counterparty_channel_id.into(),
					connection_id: connection_id.into(),
				}
			},
			RawIbcEvent::CloseConfirmChannel(close_confirm_channel) => {
				let port_id = close_confirm_channel.port_id().clone();
				let channel_id = close_confirm_channel.channel_id().clone();
				let counterparty_port_id = close_confirm_channel.counterparty_port_id().clone();
				let counterparty_channel_id =
					close_confirm_channel.counterparty_channel_id().clone();
				let connection_id = close_confirm_channel.connection_id().clone();

				Event::<T>::CloseConfirmChannel {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id: counterparty_channel_id.into(),
					connection_id: connection_id.into(),
				}
			},
			RawIbcEvent::SendPacket(send_packet) => {
				let packet_data = send_packet.packet_data().clone();
				let timeout_height = send_packet.timeout_height().clone();
				let timeout_timestamp = send_packet.timeout_timestamp().clone();
				let sequence = send_packet.sequence().clone();
				let src_port_id = send_packet.src_port_id().clone();
				let src_channel_id = send_packet.src_channel_id().clone();
				let dst_port_id = send_packet.dst_port_id().clone();
				let dst_channel_id = send_packet.dst_channel_id().clone();
				let channel_ordering = send_packet.channel_ordering().clone();
				let src_connection_id = send_packet.src_connection_id().clone();

				Event::<T>::SendPacket {
					packet_data: packet_data.into(),
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
					sequence: sequence.into(),
					src_port_id: src_port_id.into(),
					src_channel_id: src_channel_id.into(),
					dst_port_id: dst_port_id.into(),
					dst_channel_id: dst_channel_id.into(),
					channel_ordering: channel_ordering.into(),
					src_connection_id: src_connection_id.into(),
				}
			},
			RawIbcEvent::ReceivePacket(receiver_packet) => {
				let packet_data = receiver_packet.packet_data().clone();
				let timeout_height = receiver_packet.timeout_height().clone();
				let timeout_timestamp = receiver_packet.timeout_timestamp().clone();
				let sequence = receiver_packet.sequence().clone();
				let src_port_id = receiver_packet.src_port_id().clone();
				let src_channel_id = receiver_packet.src_channel_id().clone();
				let dst_port_id = receiver_packet.dst_port_id().clone();
				let dst_channel_id = receiver_packet.dst_channel_id().clone();
				let channel_ordering = receiver_packet.channel_ordering().clone();
				let dst_connection_id = receiver_packet.dst_connection_id().clone();

				Event::<T>::ReceivePacket {
					packet_data: packet_data.into(),
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
					sequence: sequence.into(),
					src_port_id: src_port_id.into(),
					src_channel_id: src_channel_id.into(),
					dst_port_id: dst_port_id.into(),
					dst_channel_id: dst_channel_id.into(),
					channel_ordering: channel_ordering.into(),
					dst_connection_id: dst_connection_id.into(),
				}
			},
			RawIbcEvent::WriteAcknowledgement(write_acknowledgement) => {
				let packet_data = write_acknowledgement.packet_data().clone();
				let timeout_height = write_acknowledgement.timeout_height().clone();
				let timeout_timestamp = write_acknowledgement.timeout_timestamp().clone();
				let sequence = write_acknowledgement.sequence().clone();
				let src_port_id = write_acknowledgement.src_port_id().clone();
				let src_channel_id = write_acknowledgement.src_channel_id().clone();
				let dst_port_id = write_acknowledgement.dst_port_id().clone();
				let dst_channel_id = write_acknowledgement.dst_channel_id().clone();
				let acknowledgement = write_acknowledgement.acknowledgement().clone();
				let dst_connection_id = write_acknowledgement.dst_connection_id().clone();

				Event::<T>::WriteAcknowledgement {
					packet_data: packet_data.into(),
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
					sequence: sequence.into(),
					src_port_id: src_port_id.into(),
					src_channel_id: src_channel_id.into(),
					dst_port_id: dst_port_id.into(),
					dst_channel_id: dst_channel_id.into(),
					acknowledgement: acknowledgement.into(),
					dst_connection_id: dst_connection_id.into(),
				}
			},
			RawIbcEvent::AcknowledgePacket(acknowledge_packet) => {
				let timeout_height = acknowledge_packet.timeout_height().clone();
				let timeout_timestamp = acknowledge_packet.timeout_timestamp().clone();
				let sequence = acknowledge_packet.sequence().clone();
				let src_port_id = acknowledge_packet.src_port_id().clone();
				let src_channel_id = acknowledge_packet.src_channel_id().clone();
				let dst_port_id = acknowledge_packet.dst_port_id().clone();
				let dst_channel_id = acknowledge_packet.dst_channel_id().clone();
				let channel_ordering = acknowledge_packet.channel_ordering().clone();
				let src_connection_id = acknowledge_packet.src_connection_id().clone();

				Event::<T>::AcknowledgePacket {
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
					sequence: sequence.into(),
					src_port_id: src_port_id.into(),
					src_channel_id: src_channel_id.into(),
					dst_port_id: dst_port_id.into(),
					dst_channel_id: dst_channel_id.into(),
					channel_ordering: channel_ordering.into(),
					src_connection_id: src_connection_id.into(),
				}
			},
			RawIbcEvent::TimeoutPacket(time_out_packet) => {
				let timeout_height = time_out_packet.timeout_height().clone();
				let timeout_timestamp = time_out_packet.timeout_timestamp().clone();
				let sequence = time_out_packet.sequence().clone();
				let src_port_id = time_out_packet.src_port_id().clone();
				let src_channel_id = time_out_packet.src_channel_id().clone();
				let dst_port_id = time_out_packet.dst_port_id().clone();
				let dst_channel_id = time_out_packet.dst_channel_id().clone();
				
				Event::<T>::TimeoutPacket {
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
					sequence: sequence.into(),
					src_port_id: src_port_id.into(),
					src_channel_id: src_channel_id.into(),
					dst_port_id: dst_port_id.into(),
					dst_channel_id: dst_channel_id.into(),
				}
			},
			RawIbcEvent::ChannelClosed(timeout_on_close_packet) => {
				let port_id = timeout_on_close_packet.port_id().clone();
				let channel_id = timeout_on_close_packet.channel_id().clone();
				let counterparty_port_id = timeout_on_close_packet.counterparty_port_id().clone();
				let maybe_counterparty_channel_id = timeout_on_close_packet.counterparty_channel_id().clone();
				let connection_id = timeout_on_close_packet.connection_id().clone();
				let channel_ordering = timeout_on_close_packet.channel_ordering().clone();
				
				Event::<T>::ChannelClosed {
					port_id: port_id.into(),
					channel_id: channel_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					maybe_counterparty_channel_id: maybe_counterparty_channel_id.map(|value| value.clone().into()),
					connection_id: connection_id.into(),
					channel_ordering: channel_ordering.into(),
				}
			},
			RawIbcEvent::AppModule(app_module) => Event::<T>::AppModule(app_module.into()),
		}
	}
}
