use crate::{
	mock::{new_test_ext, System, Test},
	Context,
};

use ibc::{downcast, events::IbcEvent};

use core::str::FromStr;

use ibc::{
	core::{
		ics02_client::{
			error::{Error, ErrorDetail},
			handler::{dispatch, ClientResult::Upgrade},
			msgs::{upgrade_client::MsgUpgradeClient, ClientMsg},
		},
		ics24_host::identifier::ClientId,
	},
	handler::HandlerOutput,
	mock::{
		client_state::{client_type as mock_client_type, MockClientState},
		consensus_state::MockConsensusState,
		header::MockHeader,
	},
	test_utils::get_dummy_account_id,
	Height,
};

#[test]
fn test_upgrade_client_ok() {
	new_test_ext().execute_with(|| {
		let client_id = ClientId::default();
		let signer = get_dummy_account_id();

		System::set_block_number(20);
		let ctx = Context::<Test>::new().with_client(&client_id, Height::new(0, 42).unwrap());

		let msg = MsgUpgradeClient {
			client_id: client_id.clone(),
			client_state: MockClientState::new(MockHeader::new(Height::new(1, 26).unwrap())).into(),
			consensus_state: MockConsensusState::new(MockHeader::new(Height::new(1, 26).unwrap()))
				.into(),
			proof_upgrade_client: Default::default(),
			proof_upgrade_consensus_state: Default::default(),
			signer,
		};

		let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg.clone()));

		match output {
			Ok(HandlerOutput { result, events: _, log }) => {
				assert!(log.is_empty());
				// Check the result
				match result {
					Upgrade(upg_res) => {
						assert_eq!(upg_res.client_id, client_id);
						assert_eq!(upg_res.client_state.as_ref().clone_into(), msg.client_state)
					},
					_ => panic!("upgrade handler result has incorrect type"),
				}
			},
			Err(err) => {
				panic!("unexpected error: {}", err);
			},
		}
	})
}

#[test]
fn test_upgrade_nonexisting_client() {
	new_test_ext().execute_with(|| {
		let client_id = ClientId::from_str("mockclient1").unwrap();
		let signer = get_dummy_account_id();

		System::set_block_number(20);
		let ctx = Context::<Test>::new().with_client(&client_id, Height::new(0, 42).unwrap());

		let msg = MsgUpgradeClient {
			client_id: ClientId::from_str("nonexistingclient").unwrap(),
			client_state: MockClientState::new(MockHeader::new(Height::new(1, 26).unwrap())).into(),
			consensus_state: MockConsensusState::new(MockHeader::new(Height::new(1, 26).unwrap()))
				.into(),
			proof_upgrade_client: Default::default(),
			proof_upgrade_consensus_state: Default::default(),
			signer,
		};

		let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg.clone()));

		match output {
			Err(Error(ErrorDetail::ClientNotFound(e), _)) => {
				assert_eq!(e.client_id, msg.client_id);
			},
			_ => {
				panic!("expected ClientNotFound error, instead got {:?}", output);
			},
		}
	})
}

#[test]
fn test_upgrade_client_low_height() {
	new_test_ext().execute_with(|| {
		let client_id = ClientId::default();
		let signer = get_dummy_account_id();

		System::set_block_number(20);
		let ctx = Context::<Test>::new().with_client(&client_id, Height::new(0, 42).unwrap());

		let msg = MsgUpgradeClient {
			client_id,
			client_state: MockClientState::new(MockHeader::new(Height::new(0, 26).unwrap())).into(),
			consensus_state: MockConsensusState::new(MockHeader::new(Height::new(0, 26).unwrap()))
				.into(),
			proof_upgrade_client: Default::default(),
			proof_upgrade_consensus_state: Default::default(),
			signer,
		};

		let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg.clone()));

		match output {
			Err(Error(ErrorDetail::LowUpgradeHeight(e), _)) => {
				assert_eq!(e.upgraded_height, Height::new(0, 42).unwrap());
				assert_eq!(
					e.client_height,
					MockClientState::try_from(msg.client_state).unwrap().latest_height()
				);
			},
			_ => {
				panic!("expected LowUpgradeHeight error, instead got {:?}", output);
			},
		}
	})
}

#[test]
fn test_upgrade_client_event() {
	new_test_ext().execute_with(|| {
		let client_id = ClientId::default();
		let signer = get_dummy_account_id();

		System::set_block_number(20);
		let ctx = Context::<Test>::new().with_client(&client_id, Height::new(0, 42).unwrap());

		let upgrade_height = Height::new(1, 26).unwrap();
		let msg = MsgUpgradeClient {
			client_id: client_id.clone(),
			client_state: MockClientState::new(MockHeader::new(upgrade_height)).into(),
			consensus_state: MockConsensusState::new(MockHeader::new(upgrade_height)).into(),
			proof_upgrade_client: Default::default(),
			proof_upgrade_consensus_state: Default::default(),
			signer,
		};

		let output = dispatch(&ctx, ClientMsg::UpgradeClient(msg)).unwrap();
		let upgrade_client_event =
			downcast!(output.events.first().unwrap() => IbcEvent::UpgradeClient).unwrap();
		assert_eq!(upgrade_client_event.client_id(), &client_id);
		assert_eq!(upgrade_client_event.client_type(), &mock_client_type());
		assert_eq!(upgrade_client_event.consensus_height(), &upgrade_height);
	})
}
