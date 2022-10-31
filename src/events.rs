use crate::*;
use ibc::{core::ics26_routing, events::IbcEvent as RawIbcEvent};
use ibc_support::Any;

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

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub enum IbcEvent {
	/// Client created event
	CreateClient { client_id: ClientId, client_type: ClientType, consensus_height: Height },
	/// Client updated event
	UpdateClient {
		client_id: ClientId,
		client_type: ClientType,
		consensus_height: Height,
		consensus_heights: Vec<Height>,
		header: Any,
	},
	/// Client upgraded event
	UpgradeClient { client_id: ClientId, client_type: ClientType, consensus_height: Height },
	/// Client misbehaviour event
	ClientMisbehaviour { client_id: ClientId, client_type: ClientType },
	/// Connection open init event
	OpenInitConnection {
		connection_id: ConnectionId,
		client_id: ClientId,
		counterparty_connection_id: Option<ConnectionId>,
		counterparty_client_id: ClientId,
	},
	/// Connection open try event
	OpenTryConnection {
		connection_id: ConnectionId,
		client_id: ClientId,
		counterparty_connection_id: Option<ConnectionId>,
		counterparty_client_id: ClientId,
	},
	/// Connection open acknowledgement event
	OpenAckConnection {
		connection_id: ConnectionId,
		client_id: ClientId,
		counterparty_connection_id: Option<ConnectionId>,
		counterparty_client_id: ClientId,
	},
	/// Connection open confirm event
	OpenConfirmConnection {
		connection_id: ConnectionId,
		client_id: ClientId,
		counterparty_connection_id: Option<ConnectionId>,
		counterparty_client_id: ClientId,
	},
	/// Channel open init event
	OpenInitChannel {
		port_id: PortId,
		channel_id: Option<ChannelId>,
		connection_id: ConnectionId,
		counterparty_port_id: PortId,
		counterparty_channel_id: Option<ChannelId>,
	},
	/// Channel open try event
	OpenTryChannel {
		port_id: PortId,
		channel_id: Option<ChannelId>,
		connection_id: ConnectionId,
		counterparty_port_id: PortId,
		counterparty_channel_id: Option<ChannelId>,
	},
	/// Channel open acknowledgement event
	OpenAckChannel {
		port_id: PortId,
		channel_id: Option<ChannelId>,
		connection_id: ConnectionId,
		counterparty_port_id: PortId,
		counterparty_channel_id: Option<ChannelId>,
	},
	/// Channel open confirm event
	OpenConfirmChannel {
		port_id: PortId,
		channel_id: Option<ChannelId>,
		connection_id: ConnectionId,
		counterparty_port_id: PortId,
		counterparty_channel_id: Option<ChannelId>,
	},
	/// Channel close init event
	CloseInitChannel {
		port_id: PortId,
		channel_id: Option<ChannelId>,
		connection_id: ConnectionId,
		counterparty_port_id: PortId,
		counterparty_channel_id: Option<ChannelId>,
	},
	/// Channel close confirm event
	CloseConfirmChannel {
		port_id: PortId,
		channel_id: Option<ChannelId>,
		connection_id: ConnectionId,
		counterparty_port_id: PortId,
		counterparty_channel_id: Option<ChannelId>,
	},
	/// Send packet event
	SendPacket { packet: Packet },
	/// Receive packet event
	ReceivePacket { packet: Packet },
	/// WriteAcknowledgement packet event
	WriteAcknowledgement { packet: Packet, ack: Vec<u8> },
	/// Acknowledgements packet event
	AcknowledgePacket { packet: Packet },
	/// Timeout packet event
	TimeoutPacket { packet: Packet },
	/// TimoutOnClose packet event
	TimeoutOnClosePacket { packet: Packet },
	/// Empty event
	Empty(Vec<u8>),
	/// App Module event
	AppModule(ModuleEvent),
}

impl From<RawIbcEvent> for IbcEvent {
	fn from(value: RawIbcEvent) -> Self {
		match value {
			RawIbcEvent::CreateClient(value) => {
				let client_id = value.client_id();
				let client_type = value.client_type();
				let consensus_height = value.consensus_height();
				IbcEvent::CreateClient {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
					consensus_height: Height::from(consensus_height.clone()),
				}
			},
			RawIbcEvent::UpdateClient(value) => {
				let client_id = value.client_id();
				let client_type = value.client_type();
				let consensus_height = value.consensus_height();
				let consensus_heights = value.consensus_heights();
				let header = value.header();
				IbcEvent::UpdateClient {
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
			RawIbcEvent::UpgradeClient(value) => {
				let client_id = value.client_id();
				let client_type = value.client_type();
				let consensus_height = value.consensus_height();
				IbcEvent::UpgradeClient {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
					consensus_height: Height::from(consensus_height.clone()),
				}
			},
			RawIbcEvent::ClientMisbehaviour(value) => {
				let client_id = value.client_id();
				let client_type = value.client_type();

				IbcEvent::ClientMisbehaviour {
					client_id: ClientId::from(client_id.clone()),
					client_type: ClientType::from(client_type.clone()),
				}
			},
			RawIbcEvent::OpenInitConnection(value) => {
				let connection_id = ConnectionId::from(value.connection_id().clone());
				let client_id = value.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = value
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));

				let counterparty_client_id = value.counterparty_client_id().clone();
				IbcEvent::OpenInitConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenTryConnection(value) => {
				let connection_id = ConnectionId::from(value.connection_id().clone());
				let client_id = value.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = value
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));

				let counterparty_client_id = value.counterparty_client_id().clone();
				IbcEvent::OpenTryConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenAckConnection(value) => {
				let connection_id = ConnectionId::from(value.connection_id().clone());
				let client_id = value.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = value
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));

				let counterparty_client_id = value.counterparty_client_id().clone();
				IbcEvent::OpenAckConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenConfirmConnection(value) => {
				let connection_id = ConnectionId::from(value.connection_id().clone());
				let client_id = value.client_id().clone();
				let counterparty_connection_id: Option<ConnectionId> = value
					.counterparty_connection_id()
					.clone()
					.map(|val| ConnectionId::from(val.clone()));

				let counterparty_client_id = value.counterparty_client_id().clone();
				IbcEvent::OpenConfirmConnection {
					connection_id,
					client_id: client_id.into(),
					counterparty_connection_id,
					counterparty_client_id: counterparty_client_id.into(),
				}
			},
			RawIbcEvent::OpenInitChannel(value) => {
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				IbcEvent::OpenInitChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenTryChannel(value) => {
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				IbcEvent::OpenTryChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenAckChannel(value) => {
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				IbcEvent::OpenAckChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::OpenConfirmChannel(value) => {
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id;
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				IbcEvent::OpenConfirmChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::CloseInitChannel(value) => {
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = Some(value.channel_id.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id;
				let counterparty_channel_id: Option<ChannelId> =
					value.counterparty_channel_id.map(|val| val.into());
				IbcEvent::CloseInitChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::CloseConfirmChannel(value) => {
				let port_id = value.port_id.clone();
				let channel_id: Option<ChannelId> = value.channel_id.clone().map(|val| val.into());
				let connection_id = value.connection_id.clone();
				let counterparty_port_id = value.counterparty_port_id.clone();
				let counterparty_channel_id: Option<ChannelId> =
					value.channel_id.map(|val| val.into());
				IbcEvent::CloseConfirmChannel {
					port_id: port_id.into(),
					channel_id,
					connection_id: connection_id.into(),
					counterparty_port_id: counterparty_port_id.into(),
					counterparty_channel_id,
				}
			},
			RawIbcEvent::SendPacket(value) => {
				let packet = value.packet;
				IbcEvent::SendPacket { packet: packet.into() }
			},
			RawIbcEvent::ReceivePacket(value) => {
				let packet = value.packet;
				IbcEvent::ReceivePacket { packet: packet.into() }
			},
			RawIbcEvent::WriteAcknowledgement(value) => {
				let packet = value.packet;
				let ack = value.ack;

				IbcEvent::WriteAcknowledgement { packet: packet.into(), ack }
			},
			RawIbcEvent::AcknowledgePacket(value) => {
				let packet = value.packet;
				IbcEvent::AcknowledgePacket { packet: packet.into() }
			},
			RawIbcEvent::TimeoutPacket(value) => {
				let packet = value.packet;
				IbcEvent::TimeoutPacket { packet: packet.into() }
			},
			RawIbcEvent::TimeoutOnClosePacket(value) => {
				let packet = value.packet;
				IbcEvent::TimeoutOnClosePacket { packet: packet.into() }
			},
			RawIbcEvent::AppModule(value) => IbcEvent::AppModule(value.into()),
		}
	}
}

impl<T: Config> From<RawIbcEvent> for Event<T> {
	fn from(event: RawIbcEvent) -> Self {
		Self::IbcEvent { event: event.into() }
	}
}
