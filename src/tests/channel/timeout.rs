pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::MsgTimeout as RawMsgTimeout, client::v1::Height as RawHeight,
	};

	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};

	/// Returns a dummy `RawMsgTimeout`, for testing only!
	/// The `height` parametrizes both the proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_timeout(
		proof_height: u64,
		timeout_height: u64,
		timeout_timestamp: u64,
	) -> RawMsgTimeout {
		RawMsgTimeout {
			packet: Some(get_dummy_raw_packet(timeout_height, timeout_timestamp)),
			proof_unreceived: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: proof_height }),
			next_sequence_recv: 1,
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tess {
	use super::test_util::get_dummy_raw_msg_timeout;
	use crate::{
		mock::{new_test_ext, Test as PalletIbcTest},
		Context,
	};
	use ibc::{
		core::{
			ics02_client::height::Height,
			ics03_connection::{
				connection::{
					ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
				},
				version::get_compatible_versions,
			},
			ics04_channel::{
				channel::{ChannelEnd, Counterparty, Order, State},
				context::ChannelReader,
				msgs::{timeout::MsgTimeout, PacketMsg},
				Version,
			},
			ics23_commitment::commitment::CommitmentPrefix,
			ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
			ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
		},
		events::IbcEvent,
		timestamp::ZERO_DURATION,
	};

	#[test]
	#[ignore]
	fn timeout_packet_processing() {
		new_test_ext().execute_with(|| {
    struct Test {
        name: String,
        ctx: Context<PalletIbcTest>,
        msg: MsgEnvelope,
        want_pass: bool,
    }

    let context = Context::<PalletIbcTest>::new();

    let msg_proof_height = 2;
    let msg_timeout_height = 5;
    let timeout_timestamp = 5;

    let client_height = Height::new(0, 2).unwrap();

    let msg = MsgTimeout::try_from(get_dummy_raw_msg_timeout(
        msg_proof_height,
        msg_timeout_height,
        timeout_timestamp,
    ))
    .unwrap();

    let packet = msg.packet.clone();
    let msg_envelope = MsgEnvelope::Packet(PacketMsg::Timeout(msg.clone()));

    let mut msg_ok = msg.clone();
    msg_ok.packet.timeout_timestamp_on_b = Default::default();
    let msg_ok_envelope = MsgEnvelope::Packet(PacketMsg::Timeout(msg_ok.clone()));

    let data = context.packet_commitment(
        &msg_ok.packet.data,
        &msg_ok.packet.timeout_height_on_b,
        &msg_ok.packet.timeout_timestamp_on_b,
    );

    let chan_end_on_a = ChannelEnd::new(
        State::Open,
        Order::default(),
        Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b.clone())),
        vec![ConnectionId::default()],
        Version::new("ics20-1".to_string()),
    );

    let mut source_ordered_channel_end = chan_end_on_a.clone();
    source_ordered_channel_end.ordering = Order::Ordered;

    let connection_end = ConnectionEnd::new(
        ConnectionState::Open,
        ClientId::default(),
        ConnectionCounterparty::new(
            ClientId::default(),
            Some(ConnectionId::default()),
            CommitmentPrefix::try_from(String::from("ibc").as_bytes().to_vec()).unwrap(),
        ),
        get_compatible_versions(),
        ZERO_DURATION,
    );

    let tests: Vec<Test> = vec![
        Test {
            name: "Processing fails because no channel exists in the context".to_string(),
            ctx: context.clone(),
            msg: msg_envelope.clone(),
            want_pass: false,
        },
        Test {
            name: "Processing fails because the client does not have a consensus state for the required height"
                .to_string(),
            ctx: context.clone().with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a.clone(),
            )
            .with_connection(ConnectionId::default(), connection_end.clone()),
            msg: msg_envelope.clone(),
            want_pass: false,
        },
        Test {
            name: "Processing fails because the proof's timeout has not been reached "
                .to_string(),
            ctx: context.clone().with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a.clone(),
            )
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), connection_end.clone()),
            msg: msg_envelope,
            want_pass: false,
        },
        Test {
            name: "Good parameters Unordered channel".to_string(),
            ctx: context.clone()
                .with_client(&ClientId::default(), client_height)
                .with_connection(ConnectionId::default(), connection_end.clone())
                .with_channel(
                    packet.port_on_a.clone(),
                    packet.chan_on_a.clone(),
                    chan_end_on_a,
                )
                .with_packet_commitment(
                    msg_ok.packet.port_on_a.clone(),
                    msg_ok.packet.chan_on_a.clone(),
                    msg_ok.packet.sequence,
                    data.clone(),
                ),
            msg: msg_ok_envelope.clone(),
            want_pass: true,
        },
        Test {
            name: "Good parameters Ordered Channel".to_string(),
            ctx: context
                .with_client(&ClientId::default(), client_height)
                .with_connection(ConnectionId::default(), connection_end)
                .with_channel(
                    packet.port_on_a.clone(),
                    packet.chan_on_a.clone(),
                    source_ordered_channel_end,
                )
                .with_packet_commitment(
                    msg_ok.packet.port_on_a.clone(),
                    msg_ok.packet.chan_on_a.clone(),
                    msg_ok.packet.sequence,
                    data,
                )
                .with_ack_sequence(
                     packet.port_on_b,
                     packet.chan_on_b,
                     1.into(),
                 ),
            msg: msg_ok_envelope,
            want_pass: true,
        },
    ]
    .into_iter()
    .collect();

    for test in tests {
        let mut test = test;
        let res = dispatch(&mut test.ctx, test.msg.clone());
        // Additionally check the events and the output objects in the result.
        match res {
            Ok(proto_output) => {
                assert!(
                    test.want_pass,
                    "TO_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                    test.name,
                    test.msg.clone(),
                    test.ctx.clone()
                );

                let events = proto_output.events;
                let src_channel_end = test
                    .ctx
                    .channel_end(&packet.port_on_a, &packet.chan_on_a)
                    .unwrap();

                if src_channel_end.order_matches(&Order::Ordered) {
                    assert_eq!(events.len(), 2);

                    assert!(matches!(events[0], IbcEvent::TimeoutPacket(_)));
                    assert!(matches!(events[1], IbcEvent::ChannelClosed(_)));
                } else {
                    assert_eq!(events.len(), 1);
                    assert!(matches!(
                        events.first().unwrap(),
                        &IbcEvent::TimeoutPacket(_)
                    ));
                }
            }
            Err(e) => {
                assert!(
                    !test.want_pass,
                    "timeout_packet: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
                    test.name,
                    test.msg.clone(),
                    test.ctx.clone(),
                    e,
                );
            }
        }
    }})
	}
}
