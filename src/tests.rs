use super::{pallet::ConsensusStates, Any, *};
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
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

#[test]
fn test_deliver_failed() {
	let alice: AccountId = AccountKeyring::Alice.into();
	let origin = Origin::signed(alice);
	let any: Any = Any { type_url: vec![1, 2, 3], value: vec![1, 2, 3] };
	let tmp: u8 = 0;
	new_test_ext().execute_with(|| {
		assert_ok!(Ibc::deliver(origin, vec![any], tmp));
	})
}

// create single grandpa client
#[test]
fn test_gp_create_client_ok() {
	new_test_ext().execute_with(|| {
		let signer = get_dummy_account_id();

		let chain_id = ChainId::new("ibc".to_string(), 0);
		let latest_height = Height::default();
		let frozen_height = Height::default();

		let gp_client_state = AnyClientState::Grandpa(
			GPClientState::new(chain_id, latest_height, frozen_height).unwrap(),
		);

		let gp_consensus_state = GPConsensusState::new(CommitmentRoot::from_bytes(&[1, 2, 3]));
		let msg = MsgCreateAnyClient::new(
			gp_client_state,
			AnyConsensusState::Grandpa(gp_consensus_state),
			signer,
		)
		.unwrap();

		let alice: AccountId = AccountKeyring::Alice.into();
		let origin = Origin::signed(alice);

		let msg = vec![msg.to_any()]
			.iter()
			.map(|message| Any {
				type_url: message.type_url.clone().as_bytes().to_vec(),
				value: message.value.clone(),
			})
			.collect();

		let output = Ibc::deliver(origin, msg, 0);

		println!("Output: {:?}", output);
	})
}

// create multi grandpa client
#[test]
fn test_multi_gp_create_client_ok() {
	new_test_ext().execute_with(|| {
		let signer = get_dummy_account_id();

		let chain_id = ChainId::new("ibc".to_string(), 0);
		let latest_height = Height::default();
		let frozen_height = Height::default();

		let gp_client_state = AnyClientState::Grandpa(
			GPClientState::new(chain_id, latest_height, frozen_height).unwrap(),
		);

		let gp_consensus_state = GPConsensusState::new(CommitmentRoot::from_bytes(&[1, 2, 3]));
		let msg = MsgCreateAnyClient::new(
			gp_client_state,
			AnyConsensusState::Grandpa(gp_consensus_state),
			signer,
		)
		.unwrap();

		let alice: AccountId = AccountKeyring::Alice.into();
		let origin = Origin::signed(alice);

		let msg = vec![msg.clone().to_any(), msg.clone().to_any(), msg.clone().to_any()]
			.iter()
			.map(|message| Any {
				type_url: message.type_url.clone().as_bytes().to_vec(),
				value: message.value.clone(),
			})
			.collect();

		let output = Ibc::deliver(origin, msg, 0);

		println!("Output: {:?}", output);
	})
}