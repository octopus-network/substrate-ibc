// use core::{default::Default, time::Duration};

// use crate::{
// 	mock::{new_test_ext, Test as PalletIbcTest, System},
// 	tests::{
// 		applications::transfer::{
// 			test::deliver as ics20_deliver,
// 			test_util::{get_dummy_msg_transfer, get_dummy_transfer_packet},
// 		},
// 		channel::{
// 			acknowledgement::test_util::get_dummy_raw_msg_ack_with_packet,
// 			chan_close_confirm::test_util::get_dummy_raw_msg_chan_close_confirm,
// 			chan_close_init::test_util::get_dummy_raw_msg_chan_close_init,
// 			chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack,
// 			chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm,
// 			chan_open_init::test_util::get_dummy_raw_msg_chan_open_init,
// 			chan_open_try::test_util::get_dummy_raw_msg_chan_open_try,
// 			recv_packet::test_util::get_dummy_raw_msg_recv_packet,
// 			timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close,
// 		},
// 		commitment::test_util::get_dummy_merkle_proof,
// 		connection::{
// 			conn_open_ack::test_util::get_dummy_raw_msg_conn_open_ack,
// 			conn_open_init::test_util::get_dummy_raw_msg_conn_open_init,
// 			conn_open_try::test_util::get_dummy_raw_msg_conn_open_try,
// 		},
// 	},
// 	Context,
// };
// use ibc::{
// 	applications::transfer::{
// 		msgs::transfer::MsgTransfer, packet::PacketData, PrefixedCoin, MODULE_ID_STR,
// 	},
// 	core::{
// 		ics02_client::msgs::{
// 			create_client::MsgCreateClient, update_client::MsgUpdateClient,
// 			upgrade_client::MsgUpgradeClient, ClientMsg,
// 		},
// 		ics03_connection::{
// 			connection::{ConnectionEnd, Counterparty as ConnCounterparty, State as ConnState},
// 			msgs::{
// 				conn_open_ack::MsgConnectionOpenAck, conn_open_init::MsgConnectionOpenInit,
// 				conn_open_try::MsgConnectionOpenTry, ConnectionMsg,
// 			},
// 			version::Version as ConnVersion,
// 		},
// 		ics04_channel::{
// 			channel::{
// 				ChannelEnd, Counterparty as ChannelCounterparty, Order as ChannelOrder,
// 				State as ChannelState,
// 			},
// 			context::ChannelReader,
// 			msgs::{
// 				acknowledgement::MsgAcknowledgement, chan_close_confirm::MsgChannelCloseConfirm,
// 				chan_close_init::MsgChannelCloseInit, chan_open_ack::MsgChannelOpenAck,
// 				chan_open_confirm::MsgChannelOpenConfirm, chan_open_init::MsgChannelOpenInit,
// 				chan_open_try::MsgChannelOpenTry, recv_packet::MsgRecvPacket,
// 				timeout_on_close::MsgTimeoutOnClose, ChannelMsg, PacketMsg,
// 			},
// 			timeout::TimeoutHeight,
// 			Version as ChannelVersion,
// 		},
// 		ics23_commitment::commitment::CommitmentPrefix,
// 		ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
// 		ics26_routing::{
// 			context::{Ics26Context, ModuleId, Router, RouterBuilder},
// 			error::Error,
// 			handler::dispatch,
// 			msgs::Ics26Envelope,
// 		},
// 	},
// 	events::IbcEvent,
// 	handler::HandlerOutputBuilder,
// 	mock::{
// 		client_state::MockClientState,
// 		consensus_state::MockConsensusState,
// 		context::{MockContext, MockRouterBuilder},
// 		header::MockHeader,
// 	},
// 	test_utils::{get_dummy_account_id, DummyTransferModule},
// 	timestamp::Timestamp,
// 	Height,
// };
// use pallet_ics20_transfer::ics20_callback::IbcTransferModule;

// #[test]
// /// These tests exercise two main paths: (1) the ability of the ICS26 routing module to dispatch
// /// messages to the correct module handler, and more importantly: (2) the ability of ICS handlers
// /// to work with the context and correctly store results (i.e., the `ClientKeeper`,
// /// `ConnectionKeeper`, and `ChannelKeeper` traits).
// fn routing_module_and_keepers() {
// 	new_test_ext().execute_with(|| {

// 	System::set_block_number(20);

// 	#[derive(Clone, Debug)]
// 	enum TestMsg {
// 		Ics26(Ics26Envelope),
// 		Ics20(MsgTransfer<PrefixedCoin>),
// 	}

// 	impl From<Ics26Envelope> for TestMsg {
// 		fn from(msg: Ics26Envelope) -> Self {
// 			Self::Ics26(msg)
// 		}
// 	}

// 	impl From<MsgTransfer<PrefixedCoin>> for TestMsg {
// 		fn from(msg: MsgTransfer<PrefixedCoin>) -> Self {
// 			Self::Ics20(msg)
// 		}
// 	}

// 	type StateCheckFn = dyn FnOnce(&Context<PalletIbcTest>) -> bool;

// 	// Test parameters
// 	struct Test {
// 		name: String,
// 		msg: TestMsg,
// 		want_pass: bool,
// 		state_check: Option<Box<StateCheckFn>>,
// 	}
// 	let default_signer = get_dummy_account_id();
// 	let client_height = 5;
// 	let start_client_height = Height::new(0, client_height).unwrap();
// 	let update_client_height = Height::new(0, 34).unwrap();
// 	let update_client_height_after_send = Height::new(0, 35).unwrap();

// 	let update_client_height_after_second_send = Height::new(0, 36).unwrap();

// 	let upgrade_client_height = Height::new(1, 2).unwrap();

// 	let upgrade_client_height_second = Height::new(1, 1).unwrap();

// 	let transfer_module_id: ModuleId = MODULE_ID_STR.parse().unwrap();

// 	// We reuse this same context across all tests. Nothing in particular needs parametrizing.

// 	let create_client_msg = MsgCreateClient::new(
// 		MockClientState::new(MockHeader::new(start_client_height)).into(),
// 		MockConsensusState::new(MockHeader::new(start_client_height)).into(),
// 		default_signer.clone(),
// 	)
// 	.unwrap();

// 	//
// 	// Connection handshake messages.
// 	//
// 	let msg_conn_init =
// 		MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();

// 	let correct_msg_conn_try = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
// 		client_height,
// 		client_height,
// 	))
// 	.unwrap();

// 	// The handler will fail to process this msg because the client height is too advanced.
// 	let incorrect_msg_conn_try = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
// 		client_height + 1,
// 		client_height + 1,
// 	))
// 	.unwrap();

// 	let msg_conn_ack = MsgConnectionOpenAck::try_from(get_dummy_raw_msg_conn_open_ack(
// 		client_height,
// 		client_height,
// 	))
// 	.unwrap();

// 	//
// 	// Channel handshake messages.
// 	//
// 	let msg_chan_init = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init()).unwrap();

// 	// The handler will fail to process this b/c the associated connection does not exist
// 	let mut incorrect_msg_chan_init = msg_chan_init.clone();
// 	incorrect_msg_chan_init.chan_end_on_a.connection_hops = vec![ConnectionId::new(590)];

// 	let msg_chan_try =
// 		MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(client_height)).unwrap();

// 	let msg_chan_ack =
// 		MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(client_height)).unwrap();

// 	let msg_chan_close_init =
// 		MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

// 	let msg_chan_close_confirm =
// 		MsgChannelCloseConfirm::try_from(get_dummy_raw_msg_chan_close_confirm(client_height))
// 			.unwrap();

// 	let msg_transfer = get_dummy_msg_transfer(Height::new(0, 35).unwrap().into(), None);
// 	let msg_transfer_two = get_dummy_msg_transfer(Height::new(0, 36).unwrap().into(), None);
// 	let msg_transfer_no_timeout = get_dummy_msg_transfer(TimeoutHeight::no_timeout(), None);
// 	let msg_transfer_no_timeout_or_timestamp = get_dummy_msg_transfer(
// 		TimeoutHeight::no_timeout(),
// 		Some(Timestamp::from_nanoseconds(0).unwrap()),
// 	);

// 	let mut msg_to_on_close =
// 		MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(36, 5)).unwrap();
// 	msg_to_on_close.packet.sequence = 2.into();
// 	msg_to_on_close.packet.timeout_height = msg_transfer_two.timeout_height;
// 	msg_to_on_close.packet.timeout_timestamp = msg_transfer_two.timeout_timestamp;

// 	let denom = msg_transfer_two.token.denom.clone();
// 	let packet_data = {
// 		let data = PacketData {
// 			token: PrefixedCoin { denom, amount: msg_transfer_two.token.amount },
// 			sender: msg_transfer_two.sender.clone(),
// 			receiver: msg_transfer_two.receiver.clone(),
// 		};
// 		serde_json::to_vec(&data).expect("PacketData's infallible Serialize impl failed")
// 	};
// 	msg_to_on_close.packet.data = packet_data;

// 	let msg_recv_packet = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(35)).unwrap();
// 	let msg_ack_packet = MsgAcknowledgement::try_from(get_dummy_raw_msg_ack_with_packet(
// 		get_dummy_transfer_packet(msg_transfer.clone(), 1u64.into()).into(),
// 		35,
// 	))
// 	.unwrap();
// 	let mut ctx = Context::<PalletIbcTest>::new();

// 	// First, create a client..
// 	let res = dispatch(
// 		&mut ctx,
// 		Ics26Envelope::Ics2Msg(ClientMsg::CreateClient(create_client_msg.clone())),
// 	);

// 	assert!(
// 		res.is_ok(),
// 		"ICS26 routing dispatch test 'client creation' failed for message {:?} with result: {:?}",
// 		create_client_msg,
// 		res
// 	);

// 	// Figure out the ID of the client that was just created.
// 	let events = res.unwrap().events;
// 	let client_id_event = events.first();
// 	assert!(client_id_event.is_some(), "There was no event generated for client creation!");
// 	let client_id = match client_id_event.unwrap() {
// 		IbcEvent::CreateClient(create_client) => create_client.client_id().clone(),
// 		event => panic!("unexpected IBC event: {:?}", event),
// 	};

// 	let tests: Vec<Test> = vec![
// 		// Test some ICS2 client functionality.
// 		Test {
// 			name: "Client update successful".to_string(),
// 			msg: Ics26Envelope::Ics2Msg(ClientMsg::UpdateClient(MsgUpdateClient {
// 				client_id: client_id.clone(),
// 				header: MockHeader::new(update_client_height)
// 					.with_timestamp(Timestamp::now())
// 					.into(),
// 				signer: default_signer.clone(),
// 			}))
// 			.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Client update fails due to stale header".to_string(),
// 			msg: Ics26Envelope::Ics2Msg(ClientMsg::UpdateClient(MsgUpdateClient {
// 				client_id: client_id.clone(),
// 				header: MockHeader::new(update_client_height).into(),
// 				signer: default_signer.clone(),
// 			}))
// 			.into(),
// 			want_pass: false,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Connection open init succeeds".to_string(),
// 			msg: Ics26Envelope::Ics3Msg(ConnectionMsg::ConnectionOpenInit(MsgConnectionOpenInit {
// 				client_id_on_a: client_id.clone(),
// 				..msg_conn_init
// 			}))
// 			.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Connection open try fails due to InvalidConsensusHeight (too high)".to_string(),
// 			msg: Ics26Envelope::Ics3Msg(ConnectionMsg::ConnectionOpenTry(Box::new(
// 				incorrect_msg_conn_try,
// 			)))
// 			.into(),
// 			want_pass: false,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Connection open try succeeds".to_string(),
// 			msg: Ics26Envelope::Ics3Msg(ConnectionMsg::ConnectionOpenTry(Box::new(
// 				correct_msg_conn_try
// 			)))
// 			.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Connection open ack succeeds".to_string(),
// 			msg: Ics26Envelope::Ics3Msg(ConnectionMsg::ConnectionOpenAck(Box::new(msg_conn_ack)))
// 				.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		// ICS04
// 		Test {
// 			name: "Channel open init succeeds".to_string(),
// 			msg: Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenInit(msg_chan_init)).into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Channel open init fail due to missing connection".to_string(),
// 			msg: Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenInit(
// 				incorrect_msg_chan_init,
// 			))
// 			.into(),
// 			want_pass: false,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Channel open try succeeds".to_string(),
// 			msg: Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenTry(msg_chan_try)).into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Channel open ack succeeds".to_string(),
// 			msg: Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenAck(msg_chan_ack)).into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Packet send".to_string(),
// 			msg: msg_transfer.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		// The client update is required in this test, because the proof associated with
// 		// msg_recv_packet has the same height as the packet TO height (see
// 		// get_dummy_raw_msg_recv_packet)
// 		Test {
// 			name: "Client update successful #2".to_string(),
// 			msg: Ics26Envelope::Ics2Msg(ClientMsg::UpdateClient(MsgUpdateClient {
// 				client_id: client_id.clone(),
// 				header: MockHeader::new(update_client_height_after_send)
// 					.with_timestamp(Timestamp::now())
// 					.into(),
// 				signer: default_signer.clone(),
// 			}))
// 			.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Receive packet".to_string(),
// 			msg: Ics26Envelope::Ics4PacketMsg(PacketMsg::RecvPacket(msg_recv_packet.clone()))
// 				.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Re-Receive packet".to_string(),
// 			msg: Ics26Envelope::Ics4PacketMsg(PacketMsg::RecvPacket(msg_recv_packet)).into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		// Ack packet
// 		Test {
// 			name: "Ack packet".to_string(),
// 			msg: Ics26Envelope::Ics4PacketMsg(PacketMsg::AckPacket(msg_ack_packet.clone())).into(),
// 			want_pass: true,
// 			state_check: Some(Box::new(move |ctx| {
// 				ctx.get_packet_commitment(
// 					&msg_ack_packet.packet.source_port,
// 					&msg_ack_packet.packet.source_channel,
// 					msg_ack_packet.packet.sequence,
// 				)
// 				.is_ok()
// 			})),
// 		},
// 		Test {
// 			name: "Packet send".to_string(),
// 			msg: msg_transfer_two.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Client update successful".to_string(),
// 			msg: Ics26Envelope::Ics2Msg(ClientMsg::UpdateClient(MsgUpdateClient {
// 				client_id: client_id.clone(),
// 				header: MockHeader::new(update_client_height_after_second_send).into(),
// 				signer: default_signer.clone(),
// 			}))
// 			.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		// Timeout packets
// 		Test {
// 			name: "Transfer message no timeout".to_string(),
// 			msg: msg_transfer_no_timeout.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Transfer message no timeout nor timestamp".to_string(),
// 			msg: msg_transfer_no_timeout_or_timestamp.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		//ICS04-close channel
// 		Test {
// 			name: "Channel close init succeeds".to_string(),
// 			msg: Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelCloseInit(msg_chan_close_init))
// 				.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Channel close confirm fails cause channel is already closed".to_string(),
// 			msg: Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelCloseConfirm(
// 				msg_chan_close_confirm,
// 			))
// 			.into(),
// 			want_pass: false,
// 			state_check: None,
// 		},
// 		//ICS04-to_on_close
// 		Test {
// 			name: "Timeout on close".to_string(),
// 			msg: Ics26Envelope::Ics4PacketMsg(PacketMsg::TimeoutOnClosePacket(msg_to_on_close))
// 				.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Client upgrade successful".to_string(),
// 			msg: Ics26Envelope::Ics2Msg(ClientMsg::UpgradeClient(MsgUpgradeClient::new(
// 				client_id.clone(),
// 				MockClientState::new(MockHeader::new(upgrade_client_height)).into(),
// 				MockConsensusState::new(MockHeader::new(upgrade_client_height)).into(),
// 				get_dummy_merkle_proof(),
// 				get_dummy_merkle_proof(),
// 				default_signer.clone(),
// 			)))
// 			.into(),
// 			want_pass: true,
// 			state_check: None,
// 		},
// 		Test {
// 			name: "Client upgrade un-successful".to_string(),
// 			msg: Ics26Envelope::Ics2Msg(ClientMsg::UpgradeClient(MsgUpgradeClient::new(
// 				client_id,
// 				MockClientState::new(MockHeader::new(upgrade_client_height_second)).into(),
// 				MockConsensusState::new(MockHeader::new(upgrade_client_height_second)).into(),
// 				get_dummy_merkle_proof(),
// 				get_dummy_merkle_proof(),
// 				default_signer,
// 			)))
// 			.into(),
// 			want_pass: false,
// 			state_check: None,
// 		},
// 	]
// 	.into_iter()
// 	.collect();

// 	for test in tests {
// 		let mut ctx = Context::<PalletIbcTest>::new();
// 		println!("ctx: {:?}", ctx);
// 		let res = match test.msg.clone() {
// 			TestMsg::Ics26(msg) => dispatch(&mut ctx, msg).map(|_| ()),
// 			TestMsg::Ics20(msg) => {
// 				let transfer_module = ctx.router_mut().get_route_mut(&transfer_module_id).unwrap();
// 				ics20_deliver(
// 					transfer_module.as_any_mut().downcast_mut::<IbcTransferModule<PalletIbcTest>>().unwrap(),
// 					&mut HandlerOutputBuilder::new(),
// 					msg,
// 				)
// 				.map(|_| ())
// 				.map_err(Error::ics04_channel)
// 			},
// 		};

// 		assert_eq!(
// 			test.want_pass,
// 			res.is_ok(),
// 			"ICS26 routing dispatch test '{}' failed for message {:?}\nwith result: {:?}",
// 			test.name,
// 			test.msg,
// 			res
// 		);

// 		if let Some(state_check) = test.state_check {
// 			assert_eq!(
// 				test.want_pass,
// 				state_check(&ctx),
// 				"ICS26 routing state check '{}' failed for message {:?}\nwith result: {:?}",
// 				test.name,
// 				test.msg,
// 				res
// 			);
// 		}
// 	}
// 	})
// }

// fn get_channel_events_ctx() -> Context<PalletIbcTest> {
// 	let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
// 	let mut ctx = Context::<PalletIbcTest>::new()
// 		.with_client(&ClientId::default(), Height::new(0, 1).unwrap())
// 		.with_connection(
// 			ConnectionId::new(0),
// 			ConnectionEnd::new(
// 				ConnState::Open,
// 				ClientId::default(),
// 				ConnCounterparty::new(
// 					ClientId::default(),
// 					Some(ConnectionId::new(0)),
// 					CommitmentPrefix::try_from(String::from("ibc").as_bytes().to_vec()).unwrap(),
// 				),
// 				vec![ConnVersion::default()],
// 				Duration::MAX,
// 			),
// 		);
// 	let _ = ctx.add_route(
// 		module_id.clone(),
// 		IbcTransferModule(core::marker::PhantomData::<PalletIbcTest>),
// 	);
// 	ctx
// }

// #[test]
// fn test_chan_open_init_event() {
// 	new_test_ext().execute_with(|| {
// 		let mut ctx = get_channel_events_ctx();

// 		let msg_chan_open_init =
// 			MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init()).unwrap();

// 		let res = dispatch(
// 			&mut ctx,
// 			Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenInit(msg_chan_open_init)),
// 		)
// 		.unwrap();

// 		assert_eq!(res.events.len(), 1);

// 		let event = res.events.first().unwrap();

// 		assert!(matches!(event, IbcEvent::OpenInitChannel(_)));
// 	})
// }

// #[test]
// fn test_chan_open_try_event() {
// 	new_test_ext().execute_with(|| {
// 		let mut ctx = get_channel_events_ctx();

// 		let msg_chan_open_try =
// 			MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(1)).unwrap();

// 		let res = dispatch(
// 			&mut ctx,
// 			Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenTry(msg_chan_open_try)),
// 		)
// 		.unwrap();

// 		assert_eq!(res.events.len(), 1);

// 		let event = res.events.first().unwrap();

// 		assert!(matches!(event, IbcEvent::OpenTryChannel(_)));
// 	})
// }

// #[test]
// fn test_chan_open_ack_event() {
// 	new_test_ext().execute_with(|| {
// 		let mut ctx = get_channel_events_ctx().with_channel(
// 			PortId::transfer(),
// 			ChannelId::default(),
// 			ChannelEnd::new(
// 				ChannelState::Init,
// 				ChannelOrder::Unordered,
// 				ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
// 				vec![ConnectionId::new(0)],
// 				ChannelVersion::default(),
// 			),
// 		);

// 		let msg_chan_open_ack =
// 			MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(1)).unwrap();

// 		let res = dispatch(
// 			&mut ctx,
// 			Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenAck(msg_chan_open_ack)),
// 		)
// 		.unwrap();

// 		assert_eq!(res.events.len(), 1);

// 		let event = res.events.first().unwrap();

// 		assert!(matches!(event, IbcEvent::OpenAckChannel(_)));
// 	})
// }

// #[test]
// fn test_chan_open_confirm_event() {
// 	new_test_ext().execute_with(|| {
// 		let mut ctx = get_channel_events_ctx().with_channel(
// 			PortId::transfer(),
// 			ChannelId::default(),
// 			ChannelEnd::new(
// 				ChannelState::TryOpen,
// 				ChannelOrder::Unordered,
// 				ChannelCounterparty::new(PortId::transfer(), Some(ChannelId::default())),
// 				vec![ConnectionId::new(0)],
// 				ChannelVersion::default(),
// 			),
// 		);

// 		let msg_chan_open_confirm =
// 			MsgChannelOpenConfirm::try_from(get_dummy_raw_msg_chan_open_confirm(1)).unwrap();

// 		let res = dispatch(
// 			&mut ctx,
// 			Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelOpenConfirm(msg_chan_open_confirm)),
// 		)
// 		.unwrap();

// 		assert_eq!(res.events.len(), 1);

// 		let event = res.events.first().unwrap();

// 		assert!(matches!(event, IbcEvent::OpenConfirmChannel(_)));
// 	})
// }

// #[test]
// fn test_chan_close_init_event() {
// 	new_test_ext().execute_with(|| {
// 		let mut ctx = get_channel_events_ctx().with_channel(
// 			PortId::transfer(),
// 			ChannelId::default(),
// 			ChannelEnd::new(
// 				ChannelState::Open,
// 				ChannelOrder::Unordered,
// 				ChannelCounterparty::new(PortId::transfer(), Some(ChannelId::default())),
// 				vec![ConnectionId::new(0)],
// 				ChannelVersion::default(),
// 			),
// 		);

// 		let msg_chan_close_init =
// 			MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

// 		let res = dispatch(
// 			&mut ctx,
// 			Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelCloseInit(msg_chan_close_init)),
// 		)
// 		.unwrap();

// 		assert_eq!(res.events.len(), 1);

// 		let event = res.events.first().unwrap();

// 		assert!(matches!(event, IbcEvent::CloseInitChannel(_)));
// 	})
// }

// #[test]
// fn test_chan_close_confirm_event() {
// 	new_test_ext().execute_with(|| {
// 		let mut ctx = get_channel_events_ctx().with_channel(
// 			PortId::transfer(),
// 			ChannelId::default(),
// 			ChannelEnd::new(
// 				ChannelState::Open,
// 				ChannelOrder::Unordered,
// 				ChannelCounterparty::new(PortId::transfer(), Some(ChannelId::default())),
// 				vec![ConnectionId::new(0)],
// 				ChannelVersion::default(),
// 			),
// 		);

// 		let msg_chan_close_confirm =
// 			MsgChannelCloseConfirm::try_from(get_dummy_raw_msg_chan_close_confirm(1)).unwrap();

// 		let res = dispatch(
// 			&mut ctx,
// 			Ics26Envelope::Ics4ChannelMsg(ChannelMsg::ChannelCloseConfirm(msg_chan_close_confirm)),
// 		)
// 		.unwrap();

// 		assert_eq!(res.events.len(), 1);

// 		let event = res.events.first().unwrap();

// 		assert!(matches!(event, IbcEvent::CloseConfirmChannel(_)));
// 	})
// }
