// pub mod channel;
// pub mod commitment;
// pub mod common;
// pub mod connection;

// #[cfg(test)]
// mod tests {
// 	use crate::{mock::*, Context};
// 	use core::str::FromStr;

// 	use crate::host::TENDERMINT_CLIENT_TYPE;
// 	use ibc::core::channel::types::channel::ChannelEnd;
// 	use ibc::core::channel::types::error::{ChannelError, PacketError};
// 	use ibc::core::client::context::types::error::ClientError;
// 	use ibc::core::connection::types::error::ConnectionError;
// 	use ibc::core::connection::types::ConnectionEnd;
// 	use ibc::core::handler::types::error::ContextError;
// 	use ibc::core::host::types::identifiers::ClientType;
// 	use ibc::core::host::types::identifiers::Sequence;
// 	use ibc::core::host::types::{
// 		identifiers::{ChannelId, ClientId, ConnectionId, PortId},
// 		path::{
// 			AckPath, ChannelEndPath, ClientConnectionPath, CommitmentPath, ConnectionPath,
// 			SeqAckPath, SeqRecvPath, SeqSendPath,
// 		},
// 	};
// 	use ibc::core::host::ExecutionContext;
// 	use ibc::core::host::ValidationContext;

// 	#[test]
// 	fn test_get_packet_commitment_state_ok() {
// 		use ibc::core::channel::types::commitment::PacketCommitment;

// 		let mut context: Context<Test> = Context::new();

// 		let range = (0..10).into_iter().collect::<Vec<u8>>();

// 		let mut port_id_vec = vec![];
// 		let mut channel_id_vec = vec![];
// 		let mut sequence_vec = vec![];

// 		for index in range.clone() {
// 			let port_id = PortId::default();
// 			port_id_vec.push(port_id);
// 			let channel_id = ChannelId::default();
// 			channel_id_vec.push(channel_id);
// 			let sequence = Sequence::from(index as u64);
// 			sequence_vec.push(sequence);
// 		}
// 		let com = PacketCommitment::from(vec![1, 2, 3]);

// 		new_test_ext().execute_with(|| {
// 			for index in 0..range.len() {
// 				let commitment_path = CommitmentPath {
// 					port_id: port_id_vec[index].clone(),
// 					channel_id: channel_id_vec[index].clone(),
// 					sequence: sequence_vec[index],
// 				};
// 				assert!(context.store_packet_commitment(&commitment_path, com.clone(),).is_ok());
// 			}
// 		})
// 	}

// 	#[test]
// 	fn test_connection_ok() {
// 		use codec::alloc::collections::HashMap;

// 		let mut input: HashMap<ConnectionId, ConnectionEnd> = HashMap::new();

// 		let connection_id0 = ConnectionId::new(0);
// 		let connection_end0 = ConnectionEnd::default();

// 		let connection_id1 = ConnectionId::new(1);
// 		let connection_end1 = ConnectionEnd::default();

// 		let connection_id2 = ConnectionId::new(2);
// 		let connection_end2 = ConnectionEnd::default();

// 		input.insert(connection_id0.clone(), connection_end0.clone());
// 		input.insert(connection_id1.clone(), connection_end1.clone());
// 		input.insert(connection_id2.clone(), connection_end2.clone());

// 		let mut context: Context<Test> = Context::new();
// 		new_test_ext().execute_with(|| {
// 			let connection_path = ConnectionPath(connection_id0.clone());
// 			assert_eq!(
// 				ExecutionContext::store_connection(
// 					&mut context,
// 					&connection_path,
// 					input.get(&connection_id0.clone()).unwrap().clone()
// 				)
// 				.is_ok(),
// 				true
// 			);
// 			let ret = ValidationContext::connection_end(&mut context, &connection_id0).unwrap();
// 			assert_eq!(ret, *input.get(&connection_id0.clone()).unwrap());
// 			let connection_path = ConnectionPath(connection_id1.clone());
// 			assert_eq!(
// 				ExecutionContext::store_connection(
// 					&mut context,
// 					&connection_path,
// 					input.get(&connection_id1.clone()).unwrap().clone()
// 				)
// 				.is_ok(),
// 				true
// 			);
// 			let connection_path = ConnectionPath(connection_id2.clone());
// 			assert_eq!(
// 				ExecutionContext::store_connection(
// 					&mut context,
// 					&connection_path,
// 					input.get(&connection_id2.clone()).unwrap().clone()
// 				)
// 				.is_ok(),
// 				true
// 			);
// 		})
// 	}

// 	#[test]
// 	fn test_connection_fail() {
// 		let connection_id = ConnectionId::new(0);
// 		let context: Context<Test> = Context::new();
// 		new_test_ext().execute_with(|| {
// 			let ret = ValidationContext::connection_end(&context, &connection_id)
// 				.unwrap_err()
// 				.to_string();
// 			assert_eq!(
// 				ret,
// 				ContextError::ConnectionError(ConnectionError::ConnectionMismatch {
// 					connection_id
// 				})
// 				.to_string()
// 			);
// 		})
// 	}

// 	#[test]
// 	fn test_connection_client_ok() {
// 		let client_id = ClientId::new(ClientType::new(TENDERMINT_CLIENT_TYPE.into()), 0).unwrap();
// 		let connection_id = ConnectionId::new(0);
// 		let mut context: Context<Test> = Context::new();

// 		let client_connection_path = ClientConnectionPath(client_id);
// 		new_test_ext().execute_with(|| {
// 			assert!(context
// 				.store_connection_to_client(&client_connection_path, connection_id)
// 				.is_ok());
// 		})
// 	}

// 	#[test]
// 	fn test_delete_packet_acknowledgement_ok() {
// 		use ibc::core::channel::types::commitment::AcknowledgementCommitment;

// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::default();
// 		let sequence = Sequence::from(0);
// 		let ack = AcknowledgementCommitment::from(vec![1, 2, 3]);

// 		let mut context: Context<Test> = Context::new();

// 		new_test_ext().execute_with(|| {
// 			let ack_path =
// 				AckPath { port_id: port_id.clone(), channel_id: channel_id.clone(), sequence };
// 			assert!(context.store_packet_acknowledgement(&ack_path, ack.clone()).is_ok());
// 			assert!(context.delete_packet_acknowledgement(&ack_path).is_ok());
// 			let result = context.get_packet_acknowledgement(&ack_path).unwrap_err().to_string();

// 			assert_eq!(
// 				result,
// 				ContextError::PacketError(PacketError::PacketAcknowledgementNotFound { sequence })
// 					.to_string()
// 			);
// 		})
// 	}

// 	#[test]
// 	fn test_get_acknowledge_state() {
// 		use ibc::core::channel::types::commitment::AcknowledgementCommitment;
// 		let range = (0..10).into_iter().collect::<Vec<u8>>();

// 		let mut port_id_vec = vec![];
// 		let mut channel_id_vec = vec![];
// 		let mut sequence_vec = vec![];
// 		let mut ack_vec = vec![];

// 		let mut context: Context<Test> = Context::new();

// 		for index in 0..range.len() {
// 			let port_id = PortId::default();
// 			port_id_vec.push(port_id);
// 			let channel_id = ChannelId::default();
// 			channel_id_vec.push(channel_id);
// 			let sequence = Sequence::from(index as u64);
// 			sequence_vec.push(sequence);
// 			ack_vec.push(AcknowledgementCommitment::from(vec![index as u8]));
// 		}

// 		new_test_ext().execute_with(|| {
// 			for index in 0..range.len() {
// 				let ack_path = AckPath {
// 					port_id: port_id_vec[index].clone(),
// 					channel_id: channel_id_vec[index].clone(),
// 					sequence: sequence_vec[index],
// 				};
// 				assert!(context
// 					.store_packet_acknowledgement(&ack_path, ack_vec[index].clone())
// 					.is_ok());
// 			}
// 		})
// 	}

// 	#[test]
// 	fn test_next_sequence_send_ok() {
// 		let sequence_id = Sequence::from(0);
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::default();
// 		let mut context: Context<Test> = Context::new();

// 		new_test_ext().execute_with(|| {
// 			let seq_send_path = SeqSendPath(port_id.clone(), channel_id.clone());
// 			assert!(context.store_next_sequence_send(&seq_send_path, sequence_id).is_ok());
// 			let result = context.get_next_sequence_send(&seq_send_path).unwrap();
// 			assert_eq!(result, sequence_id);
// 		})
// 	}

// 	#[test]
// 	fn test_store_channel_ok() {
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::default();
// 		let channel_end = ChannelEnd::default();

// 		let mut context: Context<Test> = Context::new();

// 		new_test_ext().execute_with(|| {
// 			let channel_end_path = ChannelEndPath(port_id.clone(), channel_id.clone());
// 			assert!(context.store_channel(&channel_end_path, channel_end.clone()).is_ok());
// 			let result = context.channel_end(&channel_end_path).unwrap();
// 			assert_eq!(result, channel_end);
// 		})
// 	}

// 	#[test]

// 	fn test_next_sequence_send_fail() {
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::new(0);
// 		let context: Context<Test> = Context::new();

// 		new_test_ext().execute_with(|| {
// 			let seq_send = SeqSendPath(port_id.clone(), channel_id.clone());
// 			let result = context.get_next_sequence_send(&seq_send).unwrap_err().to_string();
// 			assert_eq!(
// 				result,
// 				ContextError::PacketError(PacketError::MissingNextSendSeq { port_id, channel_id })
// 					.to_string()
// 			);
// 		})
// 	}

// 	#[test]
// 	fn test_next_sequence_recv_ok() {
// 		let sequence_id = Sequence::from(0);
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::new(0);
// 		let mut context: Context<Test> = Context::new();

// 		new_test_ext().execute_with(|| {
// 			let seq_recv = SeqRecvPath(port_id.clone(), channel_id.clone());
// 			assert!(context.store_next_sequence_recv(&seq_recv, sequence_id).is_ok());
// 			let result = context.get_next_sequence_recv(&seq_recv).unwrap();
// 			assert_eq!(result, sequence_id);
// 		})
// 	}

// 	#[test]
// 	fn test_read_channel_end_failed_by_supply_error_channel_id_port_id() {
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::default();
// 		let port_id_1 = PortId::from_str(String::from_str("port-1").unwrap().as_str()).unwrap();
// 		let channel_id_1 =
// 			ChannelId::from_str(String::from_str("channel-1").unwrap().as_str()).unwrap();

// 		let channel_end = ChannelEnd::default();

// 		let mut context: Context<Test> = Context::new();

// 		new_test_ext().execute_with(|| {
// 			let channel_end_path = ChannelEndPath(port_id.clone(), channel_id.clone());

// 			assert!(context.store_channel(&channel_end_path, channel_end.clone()).is_ok());
// 			let channel_end_path = ChannelEndPath(port_id_1.clone(), channel_id_1.clone());
// 			let result = context.channel_end(&channel_end_path).unwrap_err().to_string();
// 			assert_eq!(
// 				result,
// 				ContextError::ChannelError(ChannelError::ChannelNotFound {
// 					port_id: port_id_1.clone(),
// 					channel_id: channel_id.clone()
// 				})
// 				.to_string()
// 			);
// 			let channel_end_path = ChannelEndPath(port_id_1.clone(), channel_id_1.clone());
// 			let result = context.channel_end(&channel_end_path).unwrap_err().to_string();
// 			assert_eq!(
// 				result,
// 				ContextError::ChannelError(ChannelError::ChannelNotFound {
// 					port_id: port_id.clone(),
// 					channel_id: channel_id_1.clone()
// 				})
// 				.to_string()
// 			);

// 			let channel_end_path = ChannelEndPath(port_id_1.clone(), channel_id_1.clone());
// 			let result = context.channel_end(&channel_end_path).unwrap_err().to_string();
// 			assert_eq!(
// 				result,
// 				ContextError::ChannelError(ChannelError::ChannelNotFound {
// 					port_id: port_id_1.clone(),
// 					channel_id: channel_id_1.clone()
// 				})
// 				.to_string()
// 			);
// 		})
// 	}

// 	#[test]
// 	fn test_get_identified_channel_end() {
// 		let range = (0..10).into_iter().collect::<Vec<u8>>();

// 		let mut port_id_vec = vec![];
// 		let mut channel_id_vec = vec![];
// 		let channel_end_vec = vec![ChannelEnd::default(); range.len()];

// 		for _ in 0..range.len() {
// 			let port_id = PortId::default();
// 			port_id_vec.push(port_id);
// 			let channel_id = ChannelId::default();
// 			channel_id_vec.push(channel_id);
// 		}

// 		let mut context: Context<Test> = Context::new();
// 		new_test_ext().execute_with(|| {
// 			for index in 0..range.len() {
// 				let channel_end_path =
// 					ChannelEndPath(port_id_vec[index].clone(), channel_id_vec[index].clone());
// 				assert!(context
// 					.store_channel(&channel_end_path, channel_end_vec[index].clone())
// 					.is_ok());
// 			}
// 		})
// 	}

// 	#[test]
// 	fn test_next_sequence_recv_fail() {
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::new(0);
// 		let context: Context<Test> = Context::new();

// 		let seq_recv_path = SeqRecvPath(port_id.clone(), channel_id.clone());
// 		new_test_ext().execute_with(|| {
// 			let result = context.get_next_sequence_recv(&seq_recv_path).unwrap_err().to_string();
// 			assert_eq!(
// 				result,
// 				ContextError::PacketError(PacketError::MissingNextRecvSeq { port_id, channel_id })
// 					.to_string()
// 			);
// 		})
// 	}

// 	#[test]
// 	fn test_next_sequence_ack_ok() {
// 		let sequence_id = Sequence::from(0);
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::new(0);
// 		let mut context: Context<Test> = Context::new();

// 		let seq_ack_path = SeqAckPath(port_id, channel_id);
// 		new_test_ext().execute_with(|| {
// 			assert!(context.store_next_sequence_ack(&seq_ack_path, sequence_id).is_ok());
// 			let result = context.get_next_sequence_ack(&seq_ack_path).unwrap();
// 			assert_eq!(result, sequence_id);
// 		})
// 	}

// 	#[test]
// 	fn test_next_sequence_ack_fail() {
// 		let port_id = PortId::default();
// 		let channel_id = ChannelId::new(0);
// 		let context: Context<Test> = Context::new();

// 		let seq_ack_path = SeqAckPath(port_id.clone(), channel_id.clone());
// 		new_test_ext().execute_with(|| {
// 			let result = context.get_next_sequence_ack(&seq_ack_path).unwrap_err().to_string();
// 			assert_eq!(
// 				result,
// 				ContextError::PacketError(PacketError::MissingNextAckSeq { port_id, channel_id })
// 					.to_string()
// 			);
// 		})
// 	}
// }
