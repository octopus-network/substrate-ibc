pub mod test_util {
	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};
	use ibc_proto::ibc::core::{
		channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose, client::v1::Height as RawHeight,
	};

	#[allow(dead_code)]
	/// Returns a dummy `RawMsgTimeoutOnClose`, for testing only!
	/// The `height` parametrizes both the proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_timeout_on_close(
		height: u64,
		timeout_timestamp: u64,
	) -> RawMsgTimeoutOnClose {
		RawMsgTimeoutOnClose {
			packet: Some(get_dummy_raw_packet(height, timeout_timestamp)),
			proof_unreceived: get_dummy_proof(),
			proof_close: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: height }),
			next_sequence_recv: 1,
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::test_util::get_dummy_raw_msg_timeout_on_close;
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
				msgs::{timeout_on_close::MsgTimeoutOnClose, PacketMsg},
				Version,
			},
			ics24_host::identifier::{ClientId, ConnectionId},
			ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
		},
		events::IbcEvent,
		timestamp::ZERO_DURATION,
	};

	#[test]
	#[ignore]
	fn timeout_on_close_packet_processing() {
		new_test_ext().execute_with(|| {
    struct Test {
        name: String,
        ctx: Context<PalletIbcTest>,
        msg: MsgEnvelope,
        want_pass: bool,
    }

    let context = Context::<PalletIbcTest>::new();

    let height = 2;
    let timeout_timestamp = 5;

    let client_height = Height::new(0, 2).unwrap();

    let msg = MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(
        height,
        timeout_timestamp,
    ))
    .unwrap();
    let packet = msg.packet.clone();

    let data = context.packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );

    let chan_end_on_a = ChannelEnd::new(
        State::Open,
        Order::Ordered,
        Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b)),
        vec![ConnectionId::default()],
        Version::new("ics20-1".to_string()),
    );

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        ClientId::default(),
        ConnectionCounterparty::new(
            ClientId::default(),
            Some(ConnectionId::default()),
            Default::default(),
        ),
        get_compatible_versions(),
        ZERO_DURATION,
    );
    let msg_envelop = MsgEnvelope::Packet(PacketMsg::TimeoutOnClose(msg.clone()));

    let tests: Vec<Test> = vec![
        // todo
        // Test {
        //     name: "Processing fails because no channel exists in the context".to_string(),
        //     ctx: context.clone(),
        //     msg: msg_envelop.clone(),
        //     want_pass: false,
        // },
        // Test {
        //     name: "Processing fails no packet commitment is found".to_string(),
        //     ctx: context
        //         .clone()
        //         .with_channel(
        //             PortId::default(),
        //             ChannelId::default(),
        //             source_channel_end.clone(),
        //         )
        //         .with_connection(ConnectionId::default(), connection_end.clone()),
        //     msg: msg_envelop.clone(),
        //     want_pass: false,
        // },
        Test {
            name: "Good parameters".to_string(),
            ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a)
                    .with_channel(packet.port_on_a, packet.chan_on_a, chan_end_on_a)
                    .with_packet_commitment(
                        msg.packet.port_on_a.clone(),
                        msg.packet.chan_on_a.clone(),
                        msg.packet.sequence,
                        data,
                    ),
            msg: msg_envelop.clone(),
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
                    "TO_on_close_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                    test.name,
                    test.msg.clone(),
                    test.ctx.clone()
                );

                let events = proto_output.events;
                let src_channel_end = test
                    .ctx
                    .channel_end(&msg.packet.port_on_a, &msg.packet.chan_on_a)
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
    }
    })
	}
}
