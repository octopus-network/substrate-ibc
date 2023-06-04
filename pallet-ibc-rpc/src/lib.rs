pub mod ibc_core;

#[subxt::subxt(runtime_metadata_path = "metadata/metadata.scale")]
pub mod node_template {}

pub mod subxt_ibc_event {
	use core::time::Duration;

	use super::node_template::runtime_types::ibc::core::{
		ics02_client::events::{
			ClientMisbehaviour as SubxtClientMisbehaviour, CreateClient as SubxtCreateClient,
			UpdateClient as SubxtUpdateClient, UpgradeClient as SubxtUpgradeClient,
		},
		ics03_connection::events::{
			OpenAck as SubxtConnectionOpenAck, OpenConfirm as SubxtConnectionOpenConfirm,
			OpenInit as SubxtConnectionOpenInit, OpenTry as SubxtConnectionOpenTry,
		},
		ics04_channel::events::{
			AcknowledgePacket as SubxtAcknowledgePacket,
			// ChannelClosed as SubxtChannelClosed, // todo channelClose
			CloseConfirm as SubxtChannelCloseConfirm,
			CloseInit as SubxtChannelCloseInit,
			OpenAck as SubxtChannelOpenAck,
			OpenConfirm as SubxtChannelOpenConfirm,
			OpenInit as SubxtChannelOpenInit,
			OpenTry as SubxtChannelOpenTry,
			ReceivePacket as SubxtReceivePacket,
			SendPacket as SubxtSendPacket,
			TimeoutPacket as SubxtTimeoutPacket,
			WriteAcknowledgement as SubxtWriteAcknowledgement,
		},
	};
	use super::node_template::runtime_types::{
		self,
		ibc::core::{
			events::{
				IbcEvent as SubxtIbcEvent, MessageEvent as SubxtMessageEvent,
				ModuleEvent as SubxtModuleEvent, ModuleEventAttribute as SubxtModuleEventAttribute,
			},
			ics02_client::{
				client_type::ClientType as SubxtClientType, height::Height as SubxtHeight,
			},
			ics03_connection::{
				connection::{
					sealed::ConnectionEnd as SubxtConnectionEnd,
					Counterparty as SubxtConnectionCounterparty, State as SubxtConnectionState,
				},
				version::Version as SubxtConnectionVersion,
			},
			ics04_channel::{
				channel::{
					ChannelEnd as SubxtChannelEnd, Counterparty as SubxtChannelCounterparty,
					Order as SubxtChannelOrder, State as SubxtChannelState,
				},
				commitment::{
					AcknowledgementCommitment as SubxtAcknowledgementCommitment,
					PacketCommitment as SubxtPacketCommitment,
				},
				packet::{
					Acknowledgement as SubxtAcknowledgement, Receipt as SubxtReceipt,
					Sequence as SubxtSequence,
				},
				timeout::TimeoutHeight as SubxtTimeoutHeight,
				version::Version as SubxtChannelVersion,
			},
			ics23_commitment::commitment::CommitmentPrefix as SubxtCommitmentPrefix,
			ics24_host::identifier::{
				ChannelId as SubxtChannelId, ClientId as SubxtClientId,
				ConnectionId as SubxtConnectionId, PortId as SubxtPortId,
			},
			// router::ModuleId as SubxtModuleId,
			timestamp::Timestamp as SubxtTimestamp,
		},
	};
	use ibc_relayer_types::{
		core::{
			ics02_client::{
				client_type::ClientType,
				events::{
					Attributes as ClientAttributes, ClientMisbehaviour, CreateClient, UpdateClient,
					UpgradeClient,
				},
			},
			ics03_connection::{
				connection::{
					ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
				},
				events::{
					Attributes as ConnectionAttributes, OpenAck as ConnectionOpenAck,
					OpenConfirm as ConnectionOpenConfirm, OpenInit as ConnectionOpenInit,
					OpenTry as ConnectionOpenTry,
				},
				version::Version as ConnectionVersion,
			},
			ics04_channel::{
				channel::{
					ChannelEnd, Counterparty as ChannelCounterparty, Ordering as ChannelOrder,
					State as ChannelState,
				},
				commitment::{AcknowledgementCommitment, PacketCommitment},
				events::{
					AcknowledgePacket, CloseConfirm as ChannelCloseConfirm,
					CloseInit as ChannelCloseInit, OpenAck as ChannelOpenAck,
					OpenConfirm as ChannelOpenConfirm, OpenInit as ChannelOpenInit,
					OpenTry as ChannelOpenTry, ReceivePacket, SendPacket, TimeoutPacket,
					WriteAcknowledgement,
				},
				msgs::acknowledgement::Acknowledgement,
				packet::{Packet, Receipt, Sequence},
				timeout::TimeoutHeight,
				version::Version as ChannelVersion,
			},
			ics23_commitment::commitment::CommitmentPrefix,
			ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
		},
		events::{self, IbcEvent, ModuleEventAttribute, ModuleId},
		timestamp::Timestamp,
		Height,
	};
	use std::str::FromStr;

	// -------ics 02 client
	// client type
	impl From<SubxtClientType> for ClientType {
		fn from(value: SubxtClientType) -> Self {
			match value.0.as_ref() {
				"07-tendermint" => ClientType::Tendermint,
				"06-solomachine" => ClientType::Solomachine,
				_ => panic!("Unknown client type: {:?}", value),
			}
		}
	}
	// CreateClient
	impl From<SubxtCreateClient> for CreateClient {
		fn from(value: SubxtCreateClient) -> Self {
			let client_id = value.client_id.client_id;
			let client_type = value.client_type.client_type;
			let consensus_height = value.consensus_height.consensus_height;
			Self(ClientAttributes {
				client_id: client_id.into(),
				client_type: client_type.into(),
				consensus_height: consensus_height.into(),
			})
		}
	}

	// UpdateClient
	impl From<SubxtUpdateClient> for UpdateClient {
		fn from(value: SubxtUpdateClient) -> Self {
			let client_id = value.client_id.client_id;
			let client_type = value.client_type.client_type;
			let consensus_height = value.consensus_height.consensus_height;
			let _header = value.header.header;
			Self {
				common: ClientAttributes {
					client_id: client_id.into(),
					client_type: client_type.into(),
					consensus_height: consensus_height.into(),
				},
				// todo, this type is `Option<Box<dyn Header>>` but get Any. So At Present We set
				// None.
				header: None,
			}
		}
	}

	// UpgradeClient
	impl From<SubxtUpgradeClient> for UpgradeClient {
		fn from(value: SubxtUpgradeClient) -> Self {
			let client_id = value.client_id.client_id;
			let client_type = value.client_type.client_type;
			let consensus_height = value.consensus_height.consensus_height;
			Self(ClientAttributes {
				client_id: client_id.into(),
				client_type: client_type.into(),
				consensus_height: consensus_height.into(),
			})
		}
	}

	impl From<SubxtClientMisbehaviour> for ClientMisbehaviour {
		fn from(value: SubxtClientMisbehaviour) -> Self {
			let client_id = value.client_id.client_id;
			let client_type = value.client_type.client_type;
			// NOTICE, in ibc-rs  ClientMisbehaviour don't have consensus_height.
			let consensus_height = Height::new(0, 1).unwrap();
			Self(ClientAttributes {
				client_id: client_id.into(),
				client_type: client_type.into(),
				consensus_height,
			})
		}
	}

	// Height
	impl From<SubxtHeight> for Height {
		fn from(height: SubxtHeight) -> Self {
			Self::new(height.revision_number, height.revision_height)
				.expect("convert height: Never failed")
		}
	}

	// ------------ ics03 connection
	// ConnectionEnd
	impl From<SubxtConnectionEnd> for ConnectionEnd {
		fn from(value: SubxtConnectionEnd) -> Self {
			Self::new(
				value.state.into(),
				value.client_id.into(),
				value.counterparty.into(),
				value.versions.into_iter().map(|v| v.into()).collect(),
				Duration::new(value.delay_period_secs, value.delay_period_nanos),
			)
		}
	}

	// Counterparty
	impl From<SubxtConnectionCounterparty> for ConnectionCounterparty {
		fn from(value: SubxtConnectionCounterparty) -> Self {
			Self::new(
				value.client_id.into(),
				value.connection_id.map(|v| v.into()),
				value.prefix.bytes.try_into().expect("never failed convert prefix from vec<u8>"),
			)
		}
	}

	// State
	impl From<SubxtConnectionState> for ConnectionState {
		fn from(value: SubxtConnectionState) -> Self {
			match value {
				SubxtConnectionState::Uninitialized => Self::Uninitialized,
				SubxtConnectionState::Init => Self::Init,
				SubxtConnectionState::TryOpen => Self::TryOpen,
				SubxtConnectionState::Open => Self::Open,
			}
		}
	}

	// OpenAck
	impl From<SubxtConnectionOpenAck> for ConnectionOpenAck {
		fn from(value: SubxtConnectionOpenAck) -> Self {
			let connection_id = value.0.connection_id;
			let client_id = value.0.client_id;
			let counterparty_connection_id = value.0.counterparty_connection_id;
			let counterparty_client_id = value.0.counterparty_client_id;
			Self(ConnectionAttributes {
				connection_id: Some(connection_id.into()),
				client_id: client_id.into(),
				counterparty_connection_id: counterparty_connection_id.map(|v| v.into()),
				counterparty_client_id: counterparty_client_id.into(),
			})
		}
	}

	// OpenConfirm
	impl From<SubxtConnectionOpenConfirm> for ConnectionOpenConfirm {
		fn from(value: SubxtConnectionOpenConfirm) -> Self {
			let connection_id = value.0.connection_id;
			let client_id = value.0.client_id;
			let counterparty_connection_id = value.0.counterparty_connection_id;
			let counterparty_client_id = value.0.counterparty_client_id;
			Self(ConnectionAttributes {
				connection_id: Some(connection_id.into()),
				client_id: client_id.into(),
				counterparty_connection_id: counterparty_connection_id.map(|v| v.into()),
				counterparty_client_id: counterparty_client_id.into(),
			})
		}
	}

	// OpenInit
	impl From<SubxtConnectionOpenInit> for ConnectionOpenInit {
		fn from(value: SubxtConnectionOpenInit) -> Self {
			let connection_id = value.0.connection_id;
			let client_id = value.0.client_id;
			let counterparty_connection_id = value.0.counterparty_connection_id;
			let counterparty_client_id = value.0.counterparty_client_id;
			Self(ConnectionAttributes {
				connection_id: Some(connection_id.into()),
				client_id: client_id.into(),
				counterparty_connection_id: counterparty_connection_id.map(|v| v.into()),
				counterparty_client_id: counterparty_client_id.into(),
			})
		}
	}

	// OpenTry
	impl From<SubxtConnectionOpenTry> for ConnectionOpenTry {
		fn from(value: SubxtConnectionOpenTry) -> Self {
			let connection_id = value.0.connection_id;
			let client_id = value.0.client_id;
			let counterparty_connection_id = value.0.counterparty_connection_id;
			let counterparty_client_id = value.0.counterparty_client_id;
			Self(ConnectionAttributes {
				connection_id: Some(connection_id.into()),
				client_id: client_id.into(),
				counterparty_connection_id: counterparty_connection_id.map(|v| v.into()),
				counterparty_client_id: counterparty_client_id.into(),
			})
		}
	}

	// Version
	impl From<SubxtConnectionVersion> for ConnectionVersion {
		fn from(_value: SubxtConnectionVersion) -> Self {
			// todo(davirain)
			ConnectionVersion::default()
		}
	}

	// ------------ ibc04 channel
	// channelEnd
	impl From<SubxtChannelEnd> for ChannelEnd {
		fn from(value: SubxtChannelEnd) -> Self {
			Self {
				state: value.state.into(),
				ordering: value.ordering.into(),
				remote: value.remote.into(),
				connection_hops: value.connection_hops.into_iter().map(|v| v.into()).collect(),
				version: value.version.into(),
			}
		}
	}
	// Counterparty
	impl From<SubxtChannelCounterparty> for ChannelCounterparty {
		fn from(value: SubxtChannelCounterparty) -> Self {
			Self { port_id: value.port_id.into(), channel_id: value.channel_id.map(|v| v.into()) }
		}
	}
	// Order
	impl From<SubxtChannelOrder> for ChannelOrder {
		fn from(value: SubxtChannelOrder) -> Self {
			match value {
				SubxtChannelOrder::None => Self::Uninitialized,
				SubxtChannelOrder::Unordered => Self::Unordered,
				SubxtChannelOrder::Ordered => Self::Ordered,
			}
		}
	}
	// state
	impl From<SubxtChannelState> for ChannelState {
		fn from(value: SubxtChannelState) -> Self {
			match value {
				SubxtChannelState::Uninitialized => Self::Uninitialized,
				SubxtChannelState::Init => Self::Init,
				SubxtChannelState::TryOpen => Self::TryOpen,
				SubxtChannelState::Open => Self::Open,
				SubxtChannelState::Closed => Self::Closed,
			}
		}
	}
	// AcknowledgementCommitment
	impl From<SubxtAcknowledgementCommitment> for AcknowledgementCommitment {
		fn from(value: SubxtAcknowledgementCommitment) -> Self {
			Self::from(value.0)
		}
	}
	// packetCommitment todo
	impl From<SubxtPacketCommitment> for PacketCommitment {
		fn from(value: SubxtPacketCommitment) -> Self {
			Self::from(value.0)
		}
	}
	// AcknowledgePacket
	impl From<SubxtAcknowledgePacket> for AcknowledgePacket {
		fn from(value: SubxtAcknowledgePacket) -> Self {
			let timeout_height = value.timeout_height.timeout_height;
			let timeout_timestamp = value.timeout_timestamp.timeout_timestamp;
			let sequence = value.sequence.sequence;
			let src_port_id = value.src_port_id.src_port_id;
			let src_channel_id = value.src_channel_id.src_channel_id;
			let dst_port_id = value.dst_port_id.dst_port_id;
			let dst_channel_id = value.dst_channel_id.dst_channel_id;
			let _channel_ordering = value.channel_ordering.order;
			let _src_connection_id = value.src_connection_id.connection_id;
			Self {
				packet: Packet {
					sequence: sequence.into(),
					source_port: src_port_id.into(),
					source_channel: src_channel_id.into(),
					destination_port: dst_port_id.into(),
					destination_channel: dst_channel_id.into(),
					data: b"ack".to_vec(),
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
				},
			}
		}
	}
	// ChannelClosed (todo in ibc-rs have this data struct but in ibc-relayer-type have not this)
	// CloseConfirm
	impl From<SubxtChannelCloseConfirm> for ChannelCloseConfirm {
		fn from(value: SubxtChannelCloseConfirm) -> Self {
			let channel_id = value.channel_id.channel_id;
			let port_id = value.port_id.port_id;
			let connection_id = value.connection_id.connection_id;
			let counterparty_port_id = value.counterparty_port_id.counterparty_port_id;
			let counterparty_channel_id = value.counterparty_channel_id.counterparty_channel_id;
			Self {
				channel_id: Some(channel_id.into()),
				port_id: port_id.into(),
				connection_id: connection_id.into(),
				counterparty_port_id: counterparty_port_id.into(),
				counterparty_channel_id: Some(counterparty_channel_id.into()),
			}
		}
	}
	// CloseInit
	impl From<SubxtChannelCloseInit> for ChannelCloseInit {
		fn from(value: SubxtChannelCloseInit) -> Self {
			let channel_id = value.channel_id.channel_id;
			let port_id = value.port_id.port_id;
			let connection_id = value.connection_id.connection_id;
			let counterparty_port_id = value.counterparty_port_id.counterparty_port_id;
			let counterparty_channel_id = value.counterparty_channel_id.counterparty_channel_id;
			Self {
				channel_id: channel_id.into(),
				port_id: port_id.into(),
				connection_id: connection_id.into(),
				counterparty_port_id: counterparty_port_id.into(),
				counterparty_channel_id: Some(counterparty_channel_id.into()),
			}
		}
	}
	// OpenAck
	impl From<SubxtChannelOpenAck> for ChannelOpenAck {
		fn from(value: SubxtChannelOpenAck) -> Self {
			let channel_id = value.channel_id.channel_id;
			let port_id = value.port_id.port_id;
			let connection_id = value.connection_id.connection_id;
			let counterparty_port_id = value.counterparty_port_id.counterparty_port_id;
			let counterparty_channel_id = value.counterparty_channel_id.counterparty_channel_id;
			Self {
				channel_id: Some(channel_id.into()),
				port_id: port_id.into(),
				connection_id: connection_id.into(),
				counterparty_port_id: counterparty_port_id.into(),
				counterparty_channel_id: Some(counterparty_channel_id.into()),
			}
		}
	}
	// OpenConfirm
	impl From<SubxtChannelOpenConfirm> for ChannelOpenConfirm {
		fn from(value: SubxtChannelOpenConfirm) -> Self {
			let channel_id = value.channel_id.channel_id;
			let port_id = value.port_id.port_id;
			let connection_id = value.connection_id.connection_id;
			let counterparty_port_id = value.counterparty_port_id.counterparty_port_id;
			let counterparty_channel_id = value.counterparty_channel_id.counterparty_channel_id;
			Self {
				channel_id: Some(channel_id.into()),
				port_id: port_id.into(),
				connection_id: connection_id.into(),
				counterparty_port_id: counterparty_port_id.into(),
				counterparty_channel_id: Some(counterparty_channel_id.into()),
			}
		}
	}
	// OpenInit
	impl From<SubxtChannelOpenInit> for ChannelOpenInit {
		fn from(value: SubxtChannelOpenInit) -> Self {
			let channel_id = value.channel_id.channel_id;
			let port_id = value.port_id.port_id;
			let connection_id = value.connection_id.connection_id;
			let counterparty_port_id = value.counterparty_port_id.counterparty_port_id;
			let _version = value.version.version;
			Self {
				channel_id: Some(channel_id.into()),
				port_id: port_id.into(),
				connection_id: connection_id.into(),
				counterparty_port_id: counterparty_port_id.into(),
				counterparty_channel_id: None,
			}
		}
	}
	// OpenTry
	impl From<SubxtChannelOpenTry> for ChannelOpenTry {
		fn from(value: SubxtChannelOpenTry) -> Self {
			let channel_id = value.channel_id.channel_id;
			let port_id = value.port_id.port_id;
			let connection_id = value.connection_id.connection_id;
			let counterparty_port_id = value.counterparty_port_id.counterparty_port_id;
			let counterparty_channel_id = value.counterparty_channel_id.counterparty_channel_id;
			Self {
				channel_id: Some(channel_id.into()),
				port_id: port_id.into(),
				connection_id: connection_id.into(),
				counterparty_port_id: counterparty_port_id.into(),
				counterparty_channel_id: Some(counterparty_channel_id.into()),
			}
		}
	}
	// ReceivePacket
	impl From<SubxtReceivePacket> for ReceivePacket {
		fn from(value: SubxtReceivePacket) -> Self {
			let packet_data = value.packet_data.packet_data;
			let timeout_height = value.timeout_height.timeout_height;
			let timeout_timestamp = value.timeout_timestamp.timeout_timestamp;
			let sequence = value.sequence.sequence;
			let src_port_id = value.src_port_id.src_port_id;
			let src_channel_id = value.src_channel_id.src_channel_id;
			let dst_port_id = value.dst_port_id.dst_port_id;
			let dst_channel_id = value.dst_channel_id.dst_channel_id;
			let _channel_ordering = value.channel_ordering.order;
			let _src_connection_id = value.dst_connection_id.connection_id;
			Self {
				packet: Packet {
					sequence: sequence.into(),
					source_port: src_port_id.into(),
					source_channel: src_channel_id.into(),
					destination_port: dst_port_id.into(),
					destination_channel: dst_channel_id.into(),
					data: packet_data,
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
				},
			}
		}
	}
	// SendPacket
	impl From<SubxtSendPacket> for SendPacket {
		fn from(value: SubxtSendPacket) -> Self {
			let packet_data = value.packet_data.packet_data;
			let timeout_height = value.timeout_height.timeout_height;
			let timeout_timestamp = value.timeout_timestamp.timeout_timestamp;
			let sequence = value.sequence.sequence;
			let src_port_id = value.src_port_id.src_port_id;
			let src_channel_id = value.src_channel_id.src_channel_id;
			let dst_port_id = value.dst_port_id.dst_port_id;
			let dst_channel_id = value.dst_channel_id.dst_channel_id;
			let _channel_ordering = value.channel_ordering.order;
			let _src_connection_id = value.src_connection_id.connection_id;
			Self {
				packet: Packet {
					sequence: sequence.into(),
					source_port: src_port_id.into(),
					source_channel: src_channel_id.into(),
					destination_port: dst_port_id.into(),
					destination_channel: dst_channel_id.into(),
					data: packet_data,
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
				},
			}
		}
	}
	// TimeoutPacket
	impl From<SubxtTimeoutPacket> for TimeoutPacket {
		fn from(value: SubxtTimeoutPacket) -> Self {
			let timeout_height = value.timeout_height.timeout_height;
			let timeout_timestamp = value.timeout_timestamp.timeout_timestamp;
			let sequence = value.sequence.sequence;
			let src_port_id = value.src_port_id.src_port_id;
			let src_channel_id = value.src_channel_id.src_channel_id;
			let dst_port_id = value.dst_port_id.dst_port_id;
			let dst_channel_id = value.dst_channel_id.dst_channel_id;
			let _channel_ordering = value.channel_ordering.order;

			Self {
				packet: Packet {
					sequence: sequence.into(),
					source_port: src_port_id.into(),
					source_channel: src_channel_id.into(),
					destination_port: dst_port_id.into(),
					destination_channel: dst_channel_id.into(),
					data: b"timeout".to_vec(),
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
				},
			}
		}
	}
	// WriteAcknowledgement
	impl From<SubxtWriteAcknowledgement> for WriteAcknowledgement {
		fn from(value: SubxtWriteAcknowledgement) -> Self {
			let packet_data = value.packet_data.packet_data;
			let timeout_height = value.timeout_height.timeout_height;
			let timeout_timestamp = value.timeout_timestamp.timeout_timestamp;
			let sequence = value.sequence.sequence;
			let src_port_id = value.src_port_id.src_port_id;
			let src_channel_id = value.src_channel_id.src_channel_id;
			let dst_port_id = value.dst_port_id.dst_port_id;
			let dst_channel_id = value.dst_channel_id.dst_channel_id;
			let _src_connection_id = value.dst_connection_id.connection_id;
			let acknowledgement = value.acknowledgement.acknowledgement.0;
			Self {
				packet: Packet {
					sequence: sequence.into(),
					source_port: src_port_id.into(),
					source_channel: src_channel_id.into(),
					destination_port: dst_port_id.into(),
					destination_channel: dst_channel_id.into(),
					data: packet_data,
					timeout_height: timeout_height.into(),
					timeout_timestamp: timeout_timestamp.into(),
				},
				ack: acknowledgement,
			}
		}
	}
	// Acknowledgement todo
	impl From<SubxtAcknowledgement> for Acknowledgement {
		fn from(ack: SubxtAcknowledgement) -> Self {
			Self::from(ack.0)
		}
	}
	// Receipt todo
	impl From<SubxtReceipt> for Receipt {
		fn from(receipt: SubxtReceipt) -> Self {
			match receipt {
				SubxtReceipt::Ok => Self::Ok,
			}
		}
	}
	// Sequence
	impl From<SubxtSequence> for Sequence {
		fn from(value: SubxtSequence) -> Self {
			Sequence::from(value.0)
		}
	}
	// TimeoutHeight
	impl From<SubxtTimeoutHeight> for TimeoutHeight {
		fn from(value: SubxtTimeoutHeight) -> Self {
			match value {
				SubxtTimeoutHeight::Never => Self::Never,
				SubxtTimeoutHeight::At(v) => Self::At(v.into()),
			}
		}
	}
	// Version  todo
	impl From<SubxtChannelVersion> for ChannelVersion {
		fn from(value: SubxtChannelVersion) -> Self {
			Self(value.0)
		}
	}

	// ------- ics23 commitment
	// CommitmentPrefix todo
	impl From<SubxtCommitmentPrefix> for CommitmentPrefix {
		fn from(value: SubxtCommitmentPrefix) -> Self {
			CommitmentPrefix::try_from(value.bytes)
				.expect("converty failed because subxt commitment Prefix is empty")
		}
	}

	// -------ics24 host
	// ChannelId
	impl From<SubxtChannelId> for ChannelId {
		fn from(value: SubxtChannelId) -> Self {
			ChannelId::from_str(value.0.as_ref()).expect("convert channelId: Never failed")
		}
	}
	// clientId
	impl From<SubxtClientId> for ClientId {
		fn from(value: SubxtClientId) -> Self {
			ClientId::from_str(value.0.as_ref()).expect("convert clientId: Never failed")
		}
	}
	// connectionId
	impl From<SubxtConnectionId> for ConnectionId {
		fn from(value: SubxtConnectionId) -> Self {
			ConnectionId::from_str(value.0.as_ref()).expect("convert connectionid: Never failed")
		}
	}
	// PortId
	impl From<SubxtPortId> for PortId {
		fn from(value: SubxtPortId) -> Self {
			PortId::from_str(value.0.as_ref()).expect("convert PortId: Never failed")
		}
	}

	// // -------- ics26 routing
	// // ModuleId
	// impl From<SubxtModuleId> for ModuleId {
	// 	fn from(value: SubxtModuleId) -> Self {
	// 		ModuleId::from_str(value.0.as_ref()).expect("conver moudleid: never failed ")
	// 	}
	// }

	// --- events
	impl From<SubxtIbcEvent> for IbcEvent {
		fn from(value: SubxtIbcEvent) -> Self {
			match value {
				SubxtIbcEvent::CreateClient(value) => IbcEvent::CreateClient(value.into()),
				SubxtIbcEvent::UpdateClient(value) => IbcEvent::UpdateClient(value.into()),
				SubxtIbcEvent::UpgradeClient(value) => IbcEvent::UpgradeClient(value.into()),
				SubxtIbcEvent::ClientMisbehaviour(value) =>
					IbcEvent::ClientMisbehaviour(value.into()),
				SubxtIbcEvent::OpenInitConnection(value) =>
					IbcEvent::OpenInitConnection(value.into()),
				SubxtIbcEvent::OpenTryConnection(value) =>
					IbcEvent::OpenTryConnection(value.into()),
				SubxtIbcEvent::OpenAckConnection(value) =>
					IbcEvent::OpenAckConnection(value.into()),
				SubxtIbcEvent::OpenConfirmConnection(value) =>
					IbcEvent::OpenConfirmConnection(value.into()),
				SubxtIbcEvent::OpenInitChannel(value) => IbcEvent::OpenInitChannel(value.into()),
				SubxtIbcEvent::OpenTryChannel(value) => IbcEvent::OpenTryChannel(value.into()),
				SubxtIbcEvent::OpenAckChannel(value) => IbcEvent::OpenAckChannel(value.into()),
				SubxtIbcEvent::OpenConfirmChannel(value) =>
					IbcEvent::OpenConfirmChannel(value.into()),
				SubxtIbcEvent::CloseInitChannel(value) => IbcEvent::CloseInitChannel(value.into()),
				SubxtIbcEvent::CloseConfirmChannel(value) =>
					IbcEvent::CloseConfirmChannel(value.into()),
				SubxtIbcEvent::SendPacket(value) => IbcEvent::SendPacket(value.into()),
				SubxtIbcEvent::ReceivePacket(value) => IbcEvent::ReceivePacket(value.into()),
				SubxtIbcEvent::WriteAcknowledgement(value) =>
					IbcEvent::WriteAcknowledgement(value.into()),
				SubxtIbcEvent::AcknowledgePacket(value) =>
					IbcEvent::AcknowledgePacket(value.into()),
				SubxtIbcEvent::TimeoutPacket(value) => IbcEvent::TimeoutPacket(value.into()),
				SubxtIbcEvent::ChannelClosed(value) => IbcEvent::ChainError(format!("{:?}", value)), /* todo ibc_relayer_type::events don't have ChannelClosed variant. */
				SubxtIbcEvent::Module(value) => IbcEvent::AppModule(value.into()),
				SubxtIbcEvent::Message(value) => IbcEvent::AppModule(value.into()),
			}
		}
	}

	impl From<SubxtMessageEvent> for events::ModuleEvent {
		fn from(value: SubxtMessageEvent) -> Self {
			let kind = "message".to_string();
			let module_name =
				events::ModuleId::new(std::borrow::Cow::Borrowed(format!("{}", kind).as_ref()))
					.unwrap();
			/// The ABCI event attribute has only one attribute, with key `module`.
			/// This method gets the associated value.
			pub fn module_attribute(value: SubxtMessageEvent) -> String {
				match value {
					SubxtMessageEvent::Client => "ibc_client".to_string(),
					SubxtMessageEvent::Connection => "ibc_connection".to_string(),
					SubxtMessageEvent::Channel => "ibc_channel".to_string(),
					SubxtMessageEvent::Module(module_name) => module_name.clone(),
				}
			}
			let attributes = vec![events::ModuleEventAttribute {
				key: "module".to_string(),
				value: module_attribute(value),
			}];
			Self { kind, module_name, attributes }
		}
	}

	impl From<SubxtModuleEvent> for events::ModuleEvent {
		fn from(value: SubxtModuleEvent) -> Self {
			let kind = value.kind;
			let module_name =
				events::ModuleId::new(std::borrow::Cow::Borrowed(format!("{}", kind).as_ref()))
					.unwrap();
			let attributes = value.attributes.into_iter().map(|v| v.into()).collect();
			Self { kind, module_name, attributes }
		}
	}

	impl From<SubxtModuleEventAttribute> for ModuleEventAttribute {
		fn from(value: SubxtModuleEventAttribute) -> Self {
			Self { key: value.key, value: value.value }
		}
	}

	// ------------------- timestamp
	impl From<SubxtTimestamp> for Timestamp {
		fn from(value: SubxtTimestamp) -> Self {
			let duration = Duration::from_millis(value.time);
			Timestamp::from_nanoseconds(duration.as_nanos() as u64).ok().unwrap()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
}
