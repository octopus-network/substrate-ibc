use crate::*;
use core::borrow::Borrow;
use ibc::{core::ics26_routing, events::IbcEvent as RawIbcEvent};


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
	fn from(value: RawIbcEvent) -> Self {
		match value {
			RawIbcEvent::NewBlock(value) => Event::<T>::NewBlock { height: value.height.into() },
			RawIbcEvent::CreateClient(value) => {
				let height = value.0.height;
				let client_id = value.0.client_id;
				let client_type = value.0.client_type;
				let consensus_height = value.0.consensus_height;
				Event::<T>::CreateClient {
					height: height.into(),
					client_id: client_id.into(),
					client_type: client_type.into(),
					consensus_height: consensus_height.into(),
				}
			},
			RawIbcEvent::UpdateClient(value) => {
				let height = value.common.height;
				let client_id = value.common.client_id;
				let client_type = value.common.client_type;
				let consensus_height = value.common.consensus_height;
				Event::<T>::UpdateClient {
					height: height.into(),
					client_id: client_id.into(),
					client_type: client_type.into(),
					consensus_height: consensus_height.into(),
				}
			},
			// Upgrade client events are not currently being used
			RawIbcEvent::UpgradeClient(value) => {
				let height = value.0.height;
				let client_id = value.0.client_id;
				let client_type = value.0.client_type;
				let consensus_height = value.0.consensus_height;
				Event::<T>::UpgradeClient {
					height: height.into(),
					client_id: client_id.into(),
					client_type: client_type.into(),
					consensus_height: consensus_height.into(),
				}
			},
			RawIbcEvent::ClientMisbehaviour(value) => {
				let height = value.0.height;
				let client_id = value.0.client_id;
				let client_type = value.0.client_type;
				let consensus_height = value.0.consensus_height;
				Event::<T>::ClientMisbehaviour {
					height: height.into(),
					client_id: client_id.into(),
					client_type: client_type.into(),
					consensus_height: consensus_height.into(),
				}
			},
			RawIbcEvent::OpenInitConnection(value) => {
				let height = value.attributes().height;
				let connection_id: Option<ConnectionId> =
					value.attributes().connection_id.clone().map(|val| val.into());
				let client_id = value.attributes().client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> =
					value.attributes().counterparty_connection_id.clone().map(|val| val.into());

				let counterparty_client_id = value.attributes().counterparty_client_id.clone();
				Event::<T>::OpenInitConnection {
					height: height.into(),
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenTryConnection(value) => {
				let height = value.attributes().height;
				let connection_id: Option<ConnectionId> =
					value.attributes().connection_id.clone().map(|val| val.into());
				let client_id = value.attributes().client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> =
					value.attributes().counterparty_connection_id.clone().map(|val| val.into());

				let counterparty_client_id = value.attributes().counterparty_client_id.clone();
				Event::<T>::OpenTryConnection {
					height: height.into(),
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenAckConnection(value) => {
				let height = value.attributes().height;
				let connection_id: Option<ConnectionId> =
					value.attributes().connection_id.clone().map(|val| val.into());
				let client_id = value.attributes().client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> =
					value.attributes().counterparty_connection_id.clone().map(|val| val.into());

				let counterparty_client_id = value.attributes().counterparty_client_id.clone();
				Event::<T>::OpenAckConnection {
					height: height.into(),
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenConfirmConnection(value) => {
				let height = value.attributes().height;
				let connection_id: Option<ConnectionId> =
					value.attributes().connection_id.clone().map(|val| val.into());
				let client_id = value.attributes().client_id.clone();
				let counterparty_connection_id: Option<ConnectionId> =
					value.attributes().counterparty_connection_id.clone().map(|val| val.into());

				let counterparty_client_id = value.attributes().counterparty_client_id.clone();
				Event::<T>::OpenConfirmConnection {
					height: height.into(),
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenInitChannel(value) => {
				let height = value.height;
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				Event::<T>::OpenInitChannel {
					height: height.into(),
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenTryChannel(value) => {
				let height = value.height;
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				Event::<T>::OpenTryChannel {
					height: height.into(),
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenAckChannel(value) => {
				let height = value.height;
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				Event::<T>::OpenAckChannel {
					height: height.into(),
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenConfirmChannel(value) => {
				let height = value.height;
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id;
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				Event::<T>::OpenConfirmChannel {
					height: height.into(),
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::CloseInitChannel(value) => {
				let height = value.height;
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = Some(value.channel_id.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id;
				let counterparty_channel_id: Option<ChannelId> =
					value.counterparty_channel_id.map(|val| val.into());
				Event::<T>::CloseInitChannel {
					height: height.into(),
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::CloseConfirmChannel(value) => {
				let height = value.height;
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				Event::<T>::CloseConfirmChannel {
					height: height.into(),
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::SendPacket(value) => {
				let height = value.height;
				let packet = value.packet;
				Event::<T>::SendPacket { height: height.into(), packet: packet.into() }
			},
			RawIbcEvent::ReceivePacket(value) => {
				let height = value.height;
				let packet = value.packet;
				Event::<T>::ReceivePacket { height: height.into(), packet: packet.into() }
			},
			RawIbcEvent::WriteAcknowledgement(value) => {
				let height = value.height;
				let packet = value.packet;
				let ack = value.ack;

				Event::<T>::WriteAcknowledgement {
					height: height.into(),
					packet: packet.into(),
					ack,
				}
			},
			RawIbcEvent::AcknowledgePacket(value) => {
				let height = value.height;
				let packet = value.packet;
				Event::<T>::AcknowledgePacket { height: height.into(), packet: packet.into() }
			},
			RawIbcEvent::TimeoutPacket(value) => {
				let height = value.height;
				let packet = value.packet;
				Event::<T>::TimeoutPacket { height: height.into(), packet: packet.into() }
			},
			RawIbcEvent::TimeoutOnClosePacket(value) => {
				let height = value.height;
				let packet = value.packet;
				Event::<T>::TimeoutOnClosePacket { height: height.into(), packet: packet.into() }
			},
			RawIbcEvent::AppModule(value) => Event::<T>::AppModule(value.into()),
			RawIbcEvent::ChainError(value) => Event::<T>::ChainError(value.as_bytes().to_vec()),
		}
	}
}

// impl<T: Config> From<Vec<RawIbcEvent>> for Event<T> {
// 	fn from(events: Vec<RawIbcEvent>) -> Self {
// 		let events: Vec<Event::<T>> = events.into_iter().map(|ev| ev.into()).collect();
// 		Self::IbcEvents { events }
// 	}
// }
