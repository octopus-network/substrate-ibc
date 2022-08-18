use super::*;
use crate::{mock::*, Context};
use core::str::FromStr;

use ibc::{
	applications::transfer::{context::Ics20Context, error::Error as ICS20Error},
	clients::ics10_grandpa::{
		client_state::ClientState as GPClientState,
		consensus_state::ConsensusState as GPConsensusState, help::ValidatorSet,
	},
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_state::AnyClientState,
			client_type::ClientType,
			context::{ClientKeeper, ClientReader},
			error::Error as ICS02Error,
		},
		ics03_connection::{
			connection::{ConnectionEnd, State},
			context::{ConnectionKeeper, ConnectionReader},
			error::Error as ICS03Error,
		},
		ics04_channel::{
			channel::ChannelEnd,
			context::{ChannelKeeper, ChannelReader},
			error::Error as ICS04Error,
			packet::Sequence,
		},
		ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
	},
	timestamp::Timestamp,
	Height,
};

// test store and read client-type
#[test]
fn test_store_client_type_ok() {
	let gp_client_type = ClientType::Grandpa;
	let gp_client_id = ClientId::new(gp_client_type, 0).unwrap();

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_client_type(gp_client_id.clone(), gp_client_type).is_ok());

		let ret = context.client_type(&gp_client_id).unwrap();

		assert_eq!(ret, gp_client_type);
	})
}

#[test]
fn test_read_client_type_failed_by_supply_error_client_id() {
	let gp_client_type = ClientType::Grandpa;
	let gp_client_id = ClientId::new(gp_client_type, 0).unwrap();
	let gp_client_id_failed = ClientId::new(gp_client_type, 1).unwrap();
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_client_type(gp_client_id.clone(), gp_client_type).is_ok());

		let ret = context.client_type(&gp_client_id_failed).unwrap_err().to_string();

		assert_eq!(ret, ICS02Error::client_not_found(gp_client_id_failed).to_string());
	})
}

// test store client_state
#[test]
fn test_store_client_state_ok() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();

	let gp_client_state = GPClientState::new(
		ChainId::new("ibc".to_string(), 0),
		0,
		Commitment::default(),
		ValidatorSet::default(),
	)
	.unwrap();
	let gp_client_state = AnyClientState::Grandpa(gp_client_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_client_state(gp_client_id.clone(), gp_client_state.clone())
			.is_ok());

		// let ret = ClientReader::client_state(&context, &gp_client_id).unwrap();

		// assert_eq!(ret, gp_client_state);
	})
}

#[test]
fn test_read_client_state_failed_by_supply_error_client_id() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let gp_client_id_failed = ClientId::new(ClientType::Grandpa, 1).unwrap();
	let gp_client_state = GPClientState::new(
		ChainId::new("ibc".to_string(), 0),
		0,
		Commitment::default(),
		ValidatorSet::default(),
	)
	.unwrap();
	let gp_client_state = AnyClientState::Grandpa(gp_client_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_client_state(gp_client_id.clone(), gp_client_state.clone())
			.is_ok());

		let ret = ClientReader::client_state(&context, &gp_client_id_failed)
			.unwrap_err()
			.to_string();

		assert_eq!(ret, ICS02Error::client_not_found(gp_client_id_failed).to_string());
	})
}

#[test]
fn test_store_consensus_state_ok() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let height = Height::new(0, 1).unwrap();
	let gp_consensus_state = GPConsensusState::default();
	let consensus_state = AnyConsensusState::Grandpa(gp_consensus_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_consensus_state(gp_client_id.clone(), height, consensus_state.clone())
			.is_ok());

		// let ret = context.consensus_state(&gp_client_id, height).unwrap();

		// assert_eq!(ret, consensus_state);
	})
}

#[test]
fn test_read_consensus_state_failed_by_supply_error_client_id() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let gp_client_id_failed = ClientId::new(ClientType::Grandpa, 1).unwrap();

	let height = Height::new(0, 1).unwrap();
	let gp_consensus_state = GPConsensusState::default();
	let consensus_state = AnyConsensusState::Grandpa(gp_consensus_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(
			context
				.store_consensus_state(gp_client_id.clone(), height, consensus_state.clone())
				.is_ok(),
			true
		);

		let ret = context.consensus_state(&gp_client_id_failed, height).unwrap_err().to_string();
		assert_eq!(
			ret,
			ICS02Error::consensus_state_not_found(gp_client_id_failed.clone(), height.clone())
				.to_string()
		);
	})
}

#[test]
fn test_get_identified_any_client_state_ok() {
	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut client_state_vec = vec![];
	let mut gp_client_id_vec = vec![];

	for index in range.clone() {
		let gp_client_id = ClientId::new(ClientType::Grandpa, index as u64).unwrap();
		let gp_client_state = GPClientState::new(
			ChainId::new("ibc".to_string(), 0),
			0,
			Commitment::default(),
			ValidatorSet::default(),
		)
		.unwrap();
		let client_state = AnyClientState::Grandpa(gp_client_state);

		gp_client_id_vec.push(gp_client_id);
		client_state_vec.push(client_state);
	}

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_client_state(
					gp_client_id_vec[index].clone(),
					client_state_vec[index].clone()
				)
				.is_ok());
		}
	})
}

#[test]
fn test_get_packet_commitment_state_ok() {
	use ibc::core::ics04_channel::commitment::PacketCommitment;

	let mut context: Context<Test> = Context::new();

	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let mut sequence_vec = vec![];

	for index in range.clone() {
		let port_id = PortId::default();
		port_id_vec.push(port_id);
		let channel_id = ChannelId::from_str(&format!("channel-{}", index)).unwrap();
		channel_id_vec.push(channel_id);
		let sequence = Sequence::from(index as u64);
		sequence_vec.push(sequence);
	}
	let com = PacketCommitment::from(vec![1, 2, 3]);

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_packet_commitment(
					(
						port_id_vec[index].clone(),
						channel_id_vec[index].clone(),
						sequence_vec[index]
					),
					com.clone(),
				)
				.is_ok());
		}
	})
}

#[test]
fn test_connection_ok() {
	use codec::alloc::collections::HashMap;

	let mut input: HashMap<ConnectionId, ConnectionEnd> = HashMap::new();

	let connection_id0 = ConnectionId::new(0);
	let connection_end0 = ConnectionEnd::default();

	let connection_id1 = ConnectionId::new(1);
	let connection_end1 = ConnectionEnd::default();

	let connection_id2 = ConnectionId::new(2);
	let connection_end2 = ConnectionEnd::default();

	input.insert(connection_id0.clone(), connection_end0.clone());
	input.insert(connection_id1.clone(), connection_end1.clone());
	input.insert(connection_id2.clone(), connection_end2.clone());

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert_eq!(
			ConnectionKeeper::store_connection(
				&mut context,
				connection_id0.clone(),
				input.get(&connection_id0.clone()).unwrap()
			)
			.is_ok(),
			true
		);

		let ret = ConnectionReader::connection_end(&mut context, &connection_id0).unwrap();
		assert_eq!(ret, *input.get(&connection_id0.clone()).unwrap());

		assert_eq!(
			ConnectionKeeper::store_connection(
				&mut context,
				connection_id1.clone(),
				input.get(&connection_id1.clone()).unwrap()
			)
			.is_ok(),
			true
		);

		assert_eq!(
			ConnectionKeeper::store_connection(
				&mut context,
				connection_id2.clone(),
				input.get(&connection_id2.clone()).unwrap()
			)
			.is_ok(),
			true
		);
	})
}

#[test]
fn test_connection_fail() {
	let connection_id0 = ConnectionId::new(0);
	let context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		let ret = ConnectionReader::connection_end(&context, &connection_id0.clone())
			.unwrap_err()
			.to_string();
		assert_eq!(ret, ICS03Error::connection_mismatch(connection_id0).to_string());
	})
}

#[test]
fn test_connection_client_ok() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let connection_id = ConnectionId::new(0);
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_connection_to_client(connection_id, &gp_client_id).is_ok());
	})
}

#[test]
fn test_delete_packet_acknowledgement_ok() {
	use ibc::core::ics04_channel::commitment::AcknowledgementCommitment;

	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let ack = AcknowledgementCommitment::from(vec![1, 2, 3]);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_packet_acknowledgement(
				(port_id.clone(), channel_id.clone(), sequence),
				ack.clone()
			)
			.is_ok());

		assert!(context
			.delete_packet_acknowledgement((port_id.clone(), channel_id.clone(), sequence))
			.is_ok());

		let result = context
			.get_packet_acknowledgement(&(port_id, channel_id, sequence))
			.unwrap_err()
			.to_string();

		assert_eq!(result, ICS04Error::packet_acknowledgement_not_found(sequence).to_string());
	})
}

#[test]
fn test_get_acknowledge_state() {
	use ibc::core::ics04_channel::commitment::AcknowledgementCommitment;
	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let mut sequence_vec = vec![];
	let mut ack_vec = vec![];

	let mut value_vec = vec![];

	let mut context: Context<Test> = Context::new();

	for index in 0..range.len() {
		let port_id = PortId::from_str(&format!("transfer-{}", index)).unwrap();
		port_id_vec.push(port_id);
		let channel_id = ChannelId::from_str(&format!("channel-{}", index)).unwrap();
		channel_id_vec.push(channel_id);
		let sequence = Sequence::from(index as u64);
		sequence_vec.push(sequence);
		ack_vec.push(AcknowledgementCommitment::from(vec![index as u8]));

		value_vec.push(ChannelReader::hash(&context, vec![index as u8]).encode());
	}

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_packet_acknowledgement(
					(
						port_id_vec[index].clone(),
						channel_id_vec[index].clone(),
						sequence_vec[index]
					),
					ack_vec[index].clone()
				)
				.is_ok());
		}
	})
}

#[test]
fn test_store_connection_channles_ok() {
	let connection_id = ConnectionId::new(0);
	let port_id = PortId::from_str(String::from_str("port-0").unwrap().as_str()).unwrap();
	let channel_id = ChannelId::from_str(String::from_str("channel-0").unwrap().as_str()).unwrap();

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert!(context
			.store_connection_channels(
				connection_id.clone(),
				&(port_id.clone(), channel_id.clone())
			)
			.is_ok());

		let result = context.connection_channels(&connection_id).unwrap();

		assert_eq!(result.len(), 1);

		assert_eq!(result[0].0, port_id);
		assert_eq!(result[0].1, channel_id);
	})
}

#[test]
fn test_next_sequence_send_ok() {
	let sequence_id = Sequence::from(0);
	let port_channel = (PortId::default(), ChannelId::new(0));
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_next_sequence_send(port_channel.clone(), sequence_id).is_ok());
		let result = context.get_next_sequence_send(&port_channel).unwrap();
		assert_eq!(result, sequence_id);
	})
}

#[test]
fn test_read_conection_channels_failed_by_suppley_error_conneciton_id() {
	let connection_id = ConnectionId::new(0);
	let connection_id_failed = ConnectionId::new(1);
	let port_id = PortId::from_str(String::from_str("port-0").unwrap().as_str()).unwrap();
	let channel_id = ChannelId::from_str(String::from_str("channel-0").unwrap().as_str()).unwrap();

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert!(context
			.store_connection_channels(
				connection_id.clone(),
				&(port_id.clone(), channel_id.clone())
			)
			.is_ok());

		let result = context.connection_channels(&connection_id_failed).unwrap_err().to_string();

		assert_eq!(
			result,
			ICS04Error::connection_not_open(connection_id_failed.clone()).to_string()
		);
	})
}

#[test]
fn test_store_channel_ok() {
	let port_id = PortId::from_str(String::from_str("port-0").unwrap().as_str()).unwrap();
	let channel_id = ChannelId::from_str(String::from_str("channel-0").unwrap().as_str()).unwrap();
	let channel_end = ChannelEnd::default();

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_channel((port_id.clone(), channel_id.clone()), &channel_end)
			.is_ok());

		let result = context.channel_end(&(port_id.clone(), channel_id.clone())).unwrap();

		assert_eq!(result, channel_end);
	})
}

#[test]

fn test_next_sequence_send_fail() {
	let port_channel = (PortId::default(), ChannelId::new(0));
	let context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		let result = context.get_next_sequence_send(&port_channel.clone()).unwrap_err().to_string();
		assert_eq!(result, ICS04Error::missing_next_send_seq(port_channel).to_string());
	})
}

#[test]
fn test_next_sequence_recv_ok() {
	let sequence_id = Sequence::from(0);
	let port_channel = (PortId::default(), ChannelId::new(0));
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_next_sequence_recv(port_channel.clone(), sequence_id).is_ok());
		let result = context.get_next_sequence_recv(&port_channel).unwrap();
		assert_eq!(result, sequence_id);
	})
}

#[test]
fn test_read_channel_end_failed_by_supply_error_channel_id_port_id() {
	let port_id = PortId::from_str(String::from_str("port-0").unwrap().as_str()).unwrap();
	let channel_id = ChannelId::from_str(String::from_str("channel-0").unwrap().as_str()).unwrap();
	let port_id_1 = PortId::from_str(String::from_str("port-1").unwrap().as_str()).unwrap();
	let channel_id_1 =
		ChannelId::from_str(String::from_str("channel-1").unwrap().as_str()).unwrap();

	let channel_end = ChannelEnd::default();

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context
			.store_channel((port_id.clone(), channel_id.clone()), &channel_end)
			.is_ok());

		let result = context
			.channel_end(&(port_id_1.clone(), channel_id.clone()))
			.unwrap_err()
			.to_string();

		assert_eq!(
			result,
			ICS04Error::channel_not_found(port_id_1.clone(), channel_id.clone()).to_string()
		);

		let result = context
			.channel_end(&(port_id.clone(), channel_id_1.clone()))
			.unwrap_err()
			.to_string();

		assert_eq!(
			result,
			ICS04Error::channel_not_found(port_id.clone(), channel_id_1.clone()).to_string()
		);

		let result = context
			.channel_end(&(port_id_1.clone(), channel_id_1.clone()))
			.unwrap_err()
			.to_string();

		assert_eq!(
			result,
			ICS04Error::channel_not_found(port_id_1.clone(), channel_id_1.clone()).to_string()
		);
	})
}

#[test]
fn test_get_identified_channel_end() {
	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let channel_end_vec = vec![ChannelEnd::default(); range.len()];

	for index in 0..range.len() {
		let port_id =
			PortId::from_str(String::from_str(&format!("prot-{}", index)).unwrap().as_str())
				.unwrap();
		port_id_vec.push(port_id);
		let channel_id =
			ChannelId::from_str(String::from_str(&format!("channel-{}", index)).unwrap().as_str())
				.unwrap();
		channel_id_vec.push(channel_id);
	}

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert!(context
				.store_channel(
					(port_id_vec[index].clone(), channel_id_vec[index].clone()),
					&channel_end_vec[index].clone()
				)
				.is_ok());
		}
	})
}

#[test]
fn test_next_sequence_recv_fail() {
	let port_channel = (PortId::default(), ChannelId::new(0));
	let context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		let result = context.get_next_sequence_recv(&port_channel.clone()).unwrap_err().to_string();
		assert_eq!(result, ICS04Error::missing_next_recv_seq(port_channel).to_string());
	})
}

#[test]
fn test_next_sequence_ack_ok() {
	let sequence_id = Sequence::from(0);
	let port_channel = (PortId::default(), ChannelId::new(0));
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert!(context.store_next_sequence_ack(port_channel.clone(), sequence_id).is_ok());
		let result = context.get_next_sequence_ack(&port_channel).unwrap();
		assert_eq!(result, sequence_id);
	})
}

#[test]
fn test_next_sequence_ack_fail() {
	let port_channel = (PortId::default(), ChannelId::new(0));
	let context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		let result = context.get_next_sequence_ack(&port_channel.clone()).unwrap_err().to_string();
		assert_eq!(result, ICS04Error::missing_next_ack_seq(port_channel).to_string());
	})
}
