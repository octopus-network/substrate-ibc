use super::{pallet::ConsensusStates, Any, *};
use crate::{mock::*, routing::Context, Error};
use frame_support::{assert_noop, assert_ok};
use ibc::ics02_client::{
	context::{ClientKeeper, ClientReader},
	error::Error as ICS02Error,
};
use sp_keyring::{sr25519::Keyring, AccountKeyring};

use ibc::{
	events::IbcEvent,
	handler::HandlerOutput,
	ics02_client::{
		client_consensus::AnyConsensusState,
		client_state::AnyClientState,
		client_type::ClientType,
		handler::{dispatch as ics02_dispatch, ClientResult},
		msgs::{create_client::MsgCreateAnyClient, ClientMsg},
	},
	ics10_grandpa::{
		client_state::ClientState as GPClientState,
		consensus_state::ConsensusState as GPConsensusState,
	},
	ics23_commitment::commitment::CommitmentRoot,
	ics24_host::identifier::{ChainId, ChannelId, ClientId},
	test_utils::get_dummy_account_id,
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
fn test_read_client_type_failed() {
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

	let gp_client_state = GPClientState::new(ChainId::new("ibc".to_string(), 0), Height::default(), Height::default()).unwrap();
	let gp_client_state = AnyClientState::Grandpa(gp_client_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(context.store_client_state(gp_client_id.clone(), gp_client_state.clone()).is_ok(), true);

		let ret = context.client_state(&gp_client_id).unwrap();

		assert_eq!(ret, gp_client_state);
	})
}

#[test]
fn test_read_client_state_failed() {
	let gp_client_id = ClientId::new(ClientType::Grandpa, 0).unwrap();
	let gp_client_id_failed = ClientId::new(ClientType::Grandpa, 1).unwrap();
	let gp_client_state = GPClientState::new(ChainId::new("ibc".to_string(), 0), Height::default(), Height::default()).unwrap();
	let gp_client_state = AnyClientState::Grandpa(gp_client_state);

	let mut context: Context<Test> = Context::new();

	new_test_ext().execute_with(|| {
		assert_eq!(context.store_client_state(gp_client_id.clone(), gp_client_state.clone()).is_ok(), true);

		let ret = context.client_state(&gp_client_id_failed).unwrap_err().to_string();

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
		assert_eq!(context.store_consensus_state(gp_client_id.clone(), height, consensus_state.clone()).is_ok(), true);

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
		assert_eq!(context.store_consensus_state(gp_client_id.clone(), height, consensus_state.clone()).is_ok(), true);

		let ret = context.consensus_state(&gp_client_id_failed, height).unwrap_err().to_string();

		assert_eq!(ret, ICS02Error::consensus_state_not_found(gp_client_id_failed.clone(),  height.clone()).to_string());
	})
}