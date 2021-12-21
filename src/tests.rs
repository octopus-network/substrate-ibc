use super::{pallet::ConsensusStates, Any, *};
use crate::{mock::*, routing::Context, Error};
use core::str::FromStr;
use frame_support::{assert_noop, assert_ok};
use sp_keyring::{sr25519::Keyring, AccountKeyring};
use tendermint_proto::Protobuf;

use ibc::{
	events::IbcEvent,
	handler::HandlerOutput,
	ics02_client::{
		context::{ClientKeeper, ClientReader},
		client_consensus::AnyConsensusState,
		client_state::AnyClientState,
		client_type::ClientType,
		error::Error as ICS02Error,
		handler::{dispatch as ics02_dispatch, ClientResult},
		msgs::{create_client::MsgCreateAnyClient, ClientMsg},
	},
	ics04_channel::{
		context::{ChannelKeeper, ChannelReader},
		packet::{Sequence, Receipt},
		error::Error as ICS04Error,
	},
	ics10_grandpa::{
		client_state::ClientState as GPClientState,
		consensus_state::ConsensusState as GPConsensusState,
	},
	ics23_commitment::commitment::CommitmentRoot,
	ics24_host::identifier::{ChainId, ChannelId, ClientId, PortId},
	test_utils::get_dummy_account_id,
	timestamp::Timestamp,
	tx_msg::Msg,
	Height,
};

// for substrate-ibc
// 1. create single grandpa client
// 2. create multi grandpa client
// 3. update multi grandpa client
// 4. upgrade grandpa client

// for ibc-rs
// 5. client_type need tests
// 6. conn_open_init_msg_processing
// 7. conn_open_try_msg_processing
// 8. conn_open_confirm_msg_processing
// 9. conn_open_ack_msg_processing

// 10. ack_packet_processing
// 11. chan_open_ack_msg_processing
// 12. chan_open_confirm_msg_processing
// 13. chan_open_init_msg_processing
// 14. chan_open_try_msg_processing
// 15. send_packet_processing
// 16. recv_packet_processing
// 17. timeout_packet_processing
// 18. timeout_on_close_packet_processing
// 19. write_ack_packet_processing

// 20. routing_module_and_keepers

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
		Height::default(),
		Height::default(),
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
		Height::default(),
		Height::default(),
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
	let gp_consensus_state = GPConsensusState::new(CommitmentRoot::from_bytes(&[1, 2, 3]));
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

#[test]
fn test_read_consensus_state_failed_by_supply_error_client_id() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let gp_client_id_failed = ClientId::new(ClientType::Grandpa, 1).unwrap();

	let height = Height::default();
	let gp_consensus_state = GPConsensusState::new(CommitmentRoot::from_bytes(&[1, 2, 3]));
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
			Height::new(0, index as u64),
			Height::new(0, index as u64),
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

		let result = Pallet::<Test>::get_identified_any_client_state();

		assert_eq!(result.len(), range.len());

		for index in range {
			let (client_id, client_state) = result[index as usize].clone();
			let client_id =
				ClientId::from_str(String::from_utf8(client_id).unwrap().as_str()).unwrap();
			// println!("client_id: {:}", client_id);
			let client_state = AnyClientState::decode_vec(&*client_state).unwrap();
			// println!("client_state: {:?}", client_state);

			assert_eq!(gp_client_id_vec.iter().find(|&val| val == &client_id).is_some(), true);
			assert_eq!(client_state_vec.iter().find(|&val| val == &client_state).is_some(), true);
		}
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

		let result = Pallet::<Test>::get_packet_commitment_state();

		assert_eq!(result.len(), range.len());

		for (port_id_1, channel_id_1, sequence_1, value_1) in result  {
			let port_id_2 = PortId::from_str(String::from_utf8(port_id_1).unwrap().as_str()).unwrap();
			let channel_id_2 = ChannelId::from_str(String::from_utf8(channel_id_1).unwrap().as_str()).unwrap();
			let sequence_2 = u64::decode(&mut sequence_1.as_slice()).unwrap();
			let sequence_2 = Sequence::from(sequence_2);
			// let sequence_2 =  Sequence::from_str(String::from_utf8(sequence_1).unwrap().as_str()).unwrap();

			// assert key
			assert_eq!(port_id_vec.iter().find(|&val| val == &port_id_2).is_some(), true);
			assert_eq!(channel_id_vec.iter().find(|&val| val == &channel_id_2).is_some(), true);
			assert_eq!(sequence_vec.iter().find(|&val| val == &sequence_2).is_some(), true);

			// assert value
			assert_eq!(value_vec.iter().find(|&val| val == &value_1).is_some(), true);
		}
	})
}


#[test]
fn test_store_packet_commitent_ok() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let timestamp = Timestamp::from_nanoseconds(0).unwrap();
	let height = Height::default();
	let data = vec![1, 2, 3];
	
	let mut context: Context<Test> = Context::new();

	let value = ChannelReader::hash(&context, format!("{:?},{:?},{:?}", timestamp, height, data));

	new_test_ext().execute_with(|| {
		
		assert_eq!(context.store_packet_commitment((port_id.clone(), channel_id.clone(), sequence.clone()), timestamp, height, data).is_ok(), true);

		let result = context.get_packet_commitment(&(port_id, channel_id, sequence)).unwrap();
		
		assert_eq!(result, value);
	})
}

#[test]
fn test_read_packet_commit_failed_by_supply_error_sequence() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let sequence_failed = Sequence::from(1);

	let timestamp = Timestamp::from_nanoseconds(0).unwrap();
	let height = Height::default();
	let data = vec![1, 2, 3];
	

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		
		// store packet commitment
		assert_eq!(context.store_packet_commitment((port_id.clone(), channel_id.clone(), sequence.clone()), timestamp, height, data).is_ok(), true);

		// read packet commitment
		let result = context.get_packet_commitment(&(port_id, channel_id, sequence_failed)).unwrap_err().to_string();
		
		// assert error
		assert_eq!(result, ICS04Error::packet_commitment_not_found(sequence_failed).to_string());
	})
}

#[test]
fn test_delete_packet_commitment_ok() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let timestamp = Timestamp::from_nanoseconds(0).unwrap();
	let height = Height::default();
	let data = vec![1, 2, 3];
	
	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		
		// store packet commitment
		assert_eq!(context.store_packet_commitment((port_id.clone(), channel_id.clone(), sequence.clone()), timestamp, height, data).is_ok(), true);

		// delete packet commitment
		assert_eq!(context.delete_packet_commitment((port_id.clone(), channel_id.clone(), sequence.clone())).is_ok(), true);

		// read packet commitment
		let result = context.get_packet_commitment(&(port_id, channel_id, sequence.clone())).unwrap_err().to_string();
		
		// assert error
		assert_eq!(result, ICS04Error::packet_commitment_not_found(sequence).to_string());
	})
}


#[test]
fn test_store_packet_receipt_ok() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let receipt = Receipt::Ok;

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(context.store_packet_receipt((port_id.clone(), channel_id.clone(), sequence.clone()), receipt.clone()).is_ok(), true);

		let result = context.get_packet_receipt(&(port_id, channel_id, sequence)).unwrap();

		let result = match result {
			Receipt::Ok => "Ok",
			_ => unreachable!(),
		};

		assert_eq!(result, "Ok")
	})
}


#[test]
fn test_read_packet_receipt_failed_supply_error_sequence() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let sequence_failed = Sequence::from(1);
	let receipt = Receipt::Ok;

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(context.store_packet_receipt((port_id.clone(), channel_id.clone(), sequence.clone()), receipt.clone()).is_ok(), true);

		let result = context.get_packet_receipt(&(port_id, channel_id, sequence_failed)).unwrap_err().to_string();

		assert_eq!(result, ICS04Error::packet_receipt_not_found(sequence_failed).to_string());
	})	
}


#[test]
fn test_packet_acknowledgement_ok() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let ack = vec![1, 2, 3];

	
	let mut context: Context<Test> = Context::new();
	let value = ChannelReader::hash(&context, format!("{:?}", ack)).encode();

	new_test_ext().execute_with(|| {
		assert_eq!(
			context.store_packet_acknowledgement((port_id.clone(), channel_id.clone(), sequence.clone()), ack.clone()).is_ok(),
			true
		);

		let result = context.get_packet_acknowledgement(&(port_id, channel_id, sequence)).unwrap();

		let result = result.as_bytes().to_vec().encode();
		assert_eq!(result, value);
	})
}


#[test]
fn test_packet_acknowledgement_failed_supply_error_sequence() {
	let port_id = PortId::from_str("transfer").unwrap();
	let channel_id = ChannelId::from_str("channel-0").unwrap();
	let sequence = Sequence::from(0);
	let sequence_failed = Sequence::from(1);
	let ack = vec![1, 2, 3];

	
	let mut context: Context<Test> = Context::new();
	
	new_test_ext().execute_with(|| {
		assert_eq!(
			context.store_packet_acknowledgement((port_id.clone(), channel_id.clone(), sequence.clone()), ack.clone()).is_ok(),
			true
		);

		let result = context.get_packet_acknowledgement(&(port_id, channel_id, sequence_failed)).unwrap_err().to_string();

		assert_eq!(result, ICS04Error::packet_acknowledgement_not_found(sequence_failed).to_string());
	})
}