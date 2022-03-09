use super::*;
use crate::{mock::*, routing::Context};
use core::str::FromStr;
use ibc::{
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
		assert_eq!(context.store_client_type(gp_client_id.clone(), gp_client_type).is_ok(), true);

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
		assert_eq!(context.store_client_type(gp_client_id.clone(), gp_client_type).is_ok(), true);

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
		BlockHeader::default(),
		Commitment::default(),
		ValidatorSet::default(),
	)
	.unwrap();
	let gp_client_state = AnyClientState::Grandpa(gp_client_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(
			context
				.store_client_state(gp_client_id.clone(), gp_client_state.clone())
				.is_ok(),
			true
		);

		let ret = ClientReader::client_state(&context, &gp_client_id).unwrap();

		assert_eq!(ret, gp_client_state);
	})
}

#[test]
fn test_read_client_state_failed_by_supply_error_client_id() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let gp_client_id_failed = ClientId::new(ClientType::Grandpa, 1).unwrap();
	let gp_client_state = GPClientState::new(
		ChainId::new("ibc".to_string(), 0),
		0,
		BlockHeader::default(),
		Commitment::default(),
		ValidatorSet::default(),
	)
	.unwrap();
	let gp_client_state = AnyClientState::Grandpa(gp_client_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(
			context
				.store_client_state(gp_client_id.clone(), gp_client_state.clone())
				.is_ok(),
			true
		);

		let ret = ClientReader::client_state(&context, &gp_client_id_failed)
			.unwrap_err()
			.to_string();

		assert_eq!(ret, ICS02Error::client_not_found(gp_client_id_failed).to_string());
	})
}

#[test]
fn test_store_consensus_state_ok() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let height = Height::default();
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

		let ret = context.consensus_state(&gp_client_id, height).unwrap();

		assert_eq!(ret, consensus_state);
	})
}
// TODO
// #[test]
// fn test_read_consensus_state_failed_by_supply_error_client_id() {
// 	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
// 	let gp_client_id_failed = ClientId::new(ClientType::Grandpa, 1).unwrap();

// 	let height = Height::default();
// 	let gp_consensus_state = GPConsensusState::default();
// 	let consensus_state = AnyConsensusState::Grandpa(gp_consensus_state);

// 	let mut context: Context<Test> = Context::new();

// 	new_test_ext().execute_with(|| {
// 		assert_eq!(
// 			context
// 				.store_consensus_state(gp_client_id.clone(), height, consensus_state.clone())
// 				.is_ok(),
// 			true
// 		);

// 		let ret = context.consensus_state(&gp_client_id_failed, height).unwrap_err().to_string();

// 		assert_eq!(
// 			ret,
// 			ICS02Error::consensus_state_not_found(gp_client_id_failed.clone(), height.clone())
// 				.to_string()
// 		);
// 	})
// }

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
			BlockHeader::default(),
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
			assert_eq!(
				context
					.store_client_state(
						gp_client_id_vec[index].clone(),
						client_state_vec[index].clone()
					)
					.is_ok(),
				true
			);
		}

		// let result = Pallet::<Test>::get_identified_any_client_state();

		// assert_eq!(result.len(), range.len());

		// for index in range {
		// 	let (client_id, client_state) = result[index as usize].clone();
		// 	let client_id =
		// 		ClientId::from_str(String::from_utf8(client_id).unwrap().as_str()).unwrap();
		// 	// println!("client_id: {:}", client_id);
		// 	let client_state = AnyClientState::decode_vec(&*client_state).unwrap();
		// 	// println!("client_state: {:?}", client_state);

		// 	assert_eq!(gp_client_id_vec.iter().find(|&val| val == &client_id).is_some(), true);
		// 	assert_eq!(client_state_vec.iter().find(|&val| val == &client_state).is_some(), true);
		// }
	})
}

#[test]
fn test_get_packet_commitment_state_ok() {
	let mut context: Context<Test> = Context::new();

	let range = (0..10).into_iter().collect::<Vec<u8>>();

	let mut port_id_vec = vec![];
	let mut channel_id_vec = vec![];
	let mut sequence_vec = vec![];

	let mut timestamp_vec = vec![];
	let mut height_vec = vec![];
	let mut data_vec = vec![];

	let mut value_vec = vec![];

	for index in range.clone() {
		let port_id = PortId::from_str(&format!("port-{}", index)).unwrap();
		port_id_vec.push(port_id);
		let channel_id = ChannelId::from_str(&format!("channel-{}", index)).unwrap();
		channel_id_vec.push(channel_id);
		let sequence = Sequence::from(index as u64);
		sequence_vec.push(sequence);

		let timestamp = Timestamp::from_nanoseconds(index as u64).unwrap();
		timestamp_vec.push(timestamp);
		let height = Height::new(0, index as u64);
		height_vec.push(height);
		let data = vec![index];
		data_vec.push(data.clone());

		let input = format!("{:?},{:?},{:?}", timestamp, height, data);
		let value = ChannelReader::hash(&context, input).encode();
		value_vec.push(value);
	}

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert_eq!(
				context
					.store_packet_commitment(
						(
							port_id_vec[index].clone(),
							channel_id_vec[index].clone(),
							sequence_vec[index].clone()
						),
						timestamp_vec[index].clone(),
						height_vec[index].clone(),
						data_vec[index].clone(),
					)
					.is_ok(),
				true
			);
		}

		// let result = Pallet::<Test>::get_packet_commitment_state();

		// assert_eq!(result.len(), range.len());

		// for (port_id_1, channel_id_1, sequence_1, value_1) in result {
		// 	let port_id_2 =
		// 		PortId::from_str(String::from_utf8(port_id_1).unwrap().as_str()).unwrap();
		// 	let channel_id_2 =
		// 		ChannelId::from_str(String::from_utf8(channel_id_1).unwrap().as_str()).unwrap();
		// 	let sequence_2 = u64::decode(&mut sequence_1.as_slice()).unwrap();
		// 	let sequence_2 = Sequence::from(sequence_2);
		// 	// let sequence_2 =
		// 	// Sequence::from_str(String::from_utf8(sequence_1).unwrap().as_str()).unwrap();

		// 	// assert key
		// 	assert_eq!(port_id_vec.iter().find(|&val| val == &port_id_2).is_some(), true);
		// 	assert_eq!(channel_id_vec.iter().find(|&val| val == &channel_id_2).is_some(), true);
		// 	assert_eq!(sequence_vec.iter().find(|&val| val == &sequence_2).is_some(), true);

		// 	// assert value
		// 	assert_eq!(value_vec.iter().find(|&val| val == &value_1).is_some(), true);
		// }
	})
}

//TODO
// #[test]
// fn test_connection_ok() {
// 	let mut input: HashMap<ConnectionId, ConnectionEnd> = HashMap::new();

// 	let connection_id0 = ConnectionId::new(0);
// 	let connection_end0 = ConnectionEnd::default();

// 	let connection_id1 = ConnectionId::new(1);
// 	let mut connection_end1 = ConnectionEnd::default();
// 	connection_end1.set_state(State::from_i32(1).unwrap());

// 	let connection_id2 = ConnectionId::new(2);
// 	let mut connection_end2 = ConnectionEnd::default();
// 	connection_end2.set_state(State::from_i32(2).unwrap());

// 	input.insert(connection_id0.clone(), connection_end0.clone());
// 	input.insert(connection_id1.clone(), connection_end1.clone());
// 	input.insert(connection_id2.clone(), connection_end2.clone());

// 	let mut context: Context<Test> = Context::new();
// 	new_test_ext().execute_with(|| {
// 		assert_eq!(
// 			ConnectionKeeper::store_connection(
// 				&mut context,
// 				connection_id0.clone(),
// 				input.get(&connection_id0.clone()).unwrap()
// 			)
// 			.is_ok(),
// 			true
// 		);

// 		let ret = ConnectionReader::connection_end(&mut context, &connection_id0).unwrap();
// 		assert_eq!(ret, *input.get(&connection_id0.clone()).unwrap());

// 		assert_eq!(
// 			ConnectionKeeper::store_connection(
// 				&mut context,
// 				connection_id1.clone(),
// 				input.get(&connection_id1.clone()).unwrap()
// 			)
// 			.is_ok(),
// 			true
// 		);

// 		assert_eq!(
// 			ConnectionKeeper::store_connection(
// 				&mut context,
// 				connection_id2.clone(),
// 				input.get(&connection_id2.clone()).unwrap()
// 			)
// 			.is_ok(),
// 			true
// 		);

// 		// let result = Pallet::<Test>::get_idenfitied_connection_end();
// 		// assert_eq!(result.len(), input.len());

// 		// for (connection_id, connection_end) in result {
// 		// 	let connection_id =
// 		// 		ConnectionId::from_str(String::from_utf8(connection_id).unwrap().as_str()).unwrap();
// 		// 	let connection_end = ConnectionEnd::decode_vec(&connection_end).unwrap();
// 		// 	assert_eq!(*input.get(&connection_id).unwrap(), connection_end);
// 		// }
// 	})
// }

#[test]
fn test_connection_fail() {
	let connection_id0 = ConnectionId::new(0);
	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		let ret = ConnectionReader::connection_end(&mut context, &connection_id0.clone())
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
		assert_eq!(context.store_connection_to_client(connection_id, &gp_client_id).is_ok(), true);
	})
}

#[test]
fn test_delete_packet_acknowledgement_ok() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let ack = vec![1, 2, 3];

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(
			context
				.store_packet_acknowledgement(
					(port_id.clone(), channel_id.clone(), sequence.clone()),
					ack.clone()
				)
				.is_ok(),
			true
		);

		assert_eq!(
			context
				.delete_packet_acknowledgement((
					port_id.clone(),
					channel_id.clone(),
					sequence.clone()
				))
				.is_ok(),
			true
		);

		let result = context
			.get_packet_acknowledgement(&(port_id, channel_id, sequence))
			.unwrap_err()
			.to_string();

		assert_eq!(result, ICS04Error::packet_acknowledgement_not_found(sequence).to_string());
	})
}

#[test]
fn test_get_acknowledge_state() {
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
		ack_vec.push(vec![index as u8]);

		value_vec.push(ChannelReader::hash(&context, format!("{:?}", vec![index as u8])).encode());
	}

	new_test_ext().execute_with(|| {
		for index in 0..range.len() {
			assert_eq!(
				context
					.store_packet_acknowledgement(
						(
							port_id_vec[index].clone(),
							channel_id_vec[index].clone(),
							sequence_vec[index].clone()
						),
						ack_vec[index].clone()
					)
					.is_ok(),
				true
			);
		}

		// let result = Pallet::<Test>::get_packet_acknowledge_state();
		// assert_eq!(result.len(), range.len());

		// for (port_id_1, channel_id_1, sequence_1, value_1) in result {
		// 	let port_id_2 =
		// 		PortId::from_str(String::from_utf8(port_id_1).unwrap().as_str()).unwrap();
		// 	let channel_id_2 =
		// 		ChannelId::from_str(String::from_utf8(channel_id_1).unwrap().as_str()).unwrap();
		// 	let sequence_2 = u64::decode(&mut sequence_1.as_slice()).unwrap();
		// 	let sequence_2 = Sequence::from(sequence_2);

		// 	// assert key
		// 	assert_eq!(port_id_vec.iter().find(|&val| val == &port_id_2).is_some(), true);
		// 	assert_eq!(channel_id_vec.iter().find(|&val| val == &channel_id_2).is_some(), true);
		// 	assert_eq!(sequence_vec.iter().find(|&val| val == &sequence_2).is_some(), true);

		// 	// assert value
		// 	assert_eq!(value_vec.iter().find(|&val| val == &value_1).is_some(), true);
		// }
	})
}

#[test]
fn test_store_connection_channles_ok() {
	let connection_id = ConnectionId::new(0);
	let port_id = PortId::from_str(String::from_str("port-0").unwrap().as_str()).unwrap();
	let channel_id = ChannelId::from_str(String::from_str("channel-0").unwrap().as_str()).unwrap();

	let mut context: Context<Test> = Context::new();
	new_test_ext().execute_with(|| {
		assert_eq!(
			context
				.store_connection_channels(
					connection_id.clone(),
					&(port_id.clone(), channel_id.clone())
				)
				.is_ok(),
			true
		);

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
		assert_eq!(
			context.store_next_sequence_send(port_channel.clone(), sequence_id).is_ok(),
			true
		);
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
		assert_eq!(
			context
				.store_connection_channels(
					connection_id.clone(),
					&(port_id.clone(), channel_id.clone())
				)
				.is_ok(),
			true
		);

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
		assert_eq!(
			context
				.store_channel((port_id.clone(), channel_id.clone()), &channel_end)
				.is_ok(),
			true
		);

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
		assert_eq!(
			context.store_next_sequence_recv(port_channel.clone(), sequence_id).is_ok(),
			true
		);
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
		assert_eq!(
			context
				.store_channel((port_id.clone(), channel_id.clone()), &channel_end)
				.is_ok(),
			true
		);

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
			assert_eq!(
				context
					.store_channel(
						(port_id_vec[index].clone(), channel_id_vec[index].clone()),
						&channel_end_vec[index].clone()
					)
					.is_ok(),
				true
			);
		}

		// let result = Pallet::<Test>::get_idenfitied_channel_end();

		// assert_eq!(result.len(), range.len());

		// for (port_id_1, channel_id_1, channel_end_1) in result {
		// 	let port_id = PortId::from_str(String::from_utf8(port_id_1).unwrap().as_str()).unwrap();
		// 	let channel_id =
		// 		ChannelId::from_str(String::from_utf8(channel_id_1).unwrap().as_str()).unwrap();
		// 	let channel_end = ChannelEnd::decode_vec(&channel_end_1).unwrap();

		// 	assert_eq!(port_id_vec.iter().find(|&val| val == &port_id).is_some(), true);
		// 	assert_eq!(channel_id_vec.iter().find(|&val| val == &channel_id).is_some(), true);
		// 	assert_eq!(channel_end_vec.iter().find(|&val| val == &channel_end).is_some(), true);
		// }
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
		assert_eq!(
			context.store_next_sequence_ack(port_channel.clone(), sequence_id).is_ok(),
			true
		);
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
