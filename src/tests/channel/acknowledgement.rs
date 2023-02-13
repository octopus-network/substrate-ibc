pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::{MsgAcknowledgement as RawMsgAcknowledgement, Packet as RawPacket},
		client::v1::Height as RawHeight,
	};

	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};

	/// Returns a dummy `RawMsgAcknowledgement`, for testing only!
	/// The `height` parametrizes both the proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_acknowledgement(height: u64) -> RawMsgAcknowledgement {
		get_dummy_raw_msg_ack_with_packet(get_dummy_raw_packet(height, 1), height)
	}

	pub fn acknowledgement() -> Vec<u8> {
		use ibc::applications::transfer::acknowledgement::ConstAckSuccess;
		serde_json::to_string(&ConstAckSuccess::Success).unwrap().as_bytes().to_vec()
	}

	pub fn get_dummy_raw_msg_ack_with_packet(
		packet: RawPacket,
		height: u64,
	) -> RawMsgAcknowledgement {
		RawMsgAcknowledgement {
			packet: Some(packet),
			acknowledgement: acknowledgement(),
			proof_acked: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: height }),
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::test_util::get_dummy_raw_msg_acknowledgement;
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
				msgs::{acknowledgement::MsgAcknowledgement, PacketMsg},
				Version,
			},
			ics23_commitment::commitment::CommitmentPrefix,
			ics24_host::identifier::{ClientId, ConnectionId},
			ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
		},
		events::IbcEvent,
		timestamp::ZERO_DURATION,
	};

	#[test]
	#[ignore]
	fn ack_packet_processing() {
		new_test_ext().execute_with(|| {
    struct Test {
        name: String,
        ctx: Context<PalletIbcTest>,
        msg: MsgEnvelope,
        want_pass: bool,
    }

    let context = Context::<PalletIbcTest>::new();

    let client_height = Height::new(0, 2).unwrap();

    let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
        client_height.revision_height(),
    ))
    .unwrap();
    let packet = msg.packet.clone();
    let msg = MsgEnvelope::Packet(PacketMsg::Ack(msg));

    let data = context.packet_commitment(
        &packet.data,
        &packet.timeout_height_on_b,
        &packet.timeout_timestamp_on_b,
    );

    let source_channel_end = ChannelEnd::new(
        State::Open,
        Order::default(),
        Counterparty::new(
            packet.port_on_b.clone(),
            Some(packet.chan_on_b.clone()),
        ),
        vec![ConnectionId::default()],
        Version::new("ics20-1".to_string()),
    );

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
        // Test {
        //     name: "Processing fails because no channel exists in the context".to_string(),
        //     ctx: context.clone(),
        //     msg: msg.clone(),
        //     want_pass: false,
        // },
        Test {
            name: "Good parameters".to_string(),
            ctx: context
                .with_client(&ClientId::default(), client_height)
                .with_connection(ConnectionId::default(), connection_end)
                .with_channel(
                    packet.port_on_a.clone(),
                    packet.chan_on_a.clone(),
                    source_channel_end,
                )
                .with_packet_commitment(
                    packet.port_on_a,
                    packet.chan_on_a,
                    packet.sequence,
                    data,
                ) //with_ack_sequence required for ordered channels
                .with_ack_sequence(
                    packet.port_on_b,
                    packet.chan_on_b,
                    1.into(),
                ),
            msg,
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
                    "ack_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                    test.name,
                    test.msg.clone(),
                    test.ctx.clone()
                );

                assert!(!proto_output.events.is_empty()); // Some events must exist.

                for e in proto_output.events.iter() {
                    assert!(matches!(e, &IbcEvent::AcknowledgePacket(_)));
                }
            }
            Err(e) => {
                assert!(
                    !test.want_pass,
                    "ack_packet: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
                    test.name,
                    test.msg,
                    test.ctx.clone(),
                    e,
                );
            }
        }
    }
})
	}
}
