pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::MsgRecvPacket as RawMsgRecvPacket, client::v1::Height as RawHeight,
	};

	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};
	use core::{ops::Add, time::Duration};
	use ibc::timestamp::Timestamp;

	/// Returns a dummy `RawMsgRecvPacket`, for testing only! The `height` parametrizes both the
	/// proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_recv_packet(height: u64) -> RawMsgRecvPacket {
		let timestamp = Timestamp::now().add(Duration::from_secs(9));
		RawMsgRecvPacket {
			packet: Some(get_dummy_raw_packet(height, timestamp.unwrap().nanoseconds())),
			proof_commitment: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: height }),
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tests {

	use super::test_util::get_dummy_raw_msg_recv_packet;
	use crate::{
		mock::{new_test_ext, System, Test as PalletIbcTest},
		tests::common::get_dummy_account_id,
		Context,
	};
	use ibc::{
		core::{
			ics03_connection::{
				connection::{
					ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
				},
				version::get_compatible_versions,
			},
			ics04_channel::{
				channel::{ChannelEnd, Counterparty, Order, State},
				msgs::{recv_packet::MsgRecvPacket, PacketMsg},
				packet::Packet,
				Version,
			},
			ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
			ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
		},
		events::IbcEvent,
		timestamp::{Timestamp, ZERO_DURATION},
		Height,
	};

	#[test]
	#[ignore]
	fn recv_packet_processing() {
		new_test_ext().execute_with(|| {
    struct Test {
        name: String,
        ctx: Context<PalletIbcTest>,
        msg: MsgEnvelope,
        want_pass: bool,
    }

    System::set_block_number(20);


    let context = Context::<PalletIbcTest>::new();

    let host_height = Height::new(0, 20).unwrap();

    let client_height = host_height.increment();

    System::set_block_number(client_height.revision_height() as u32);

    let msg = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(
        client_height.revision_height(),
    ))
    .unwrap();

    let packet = msg.packet.clone();

    let packet_old = Packet {
        sequence: 1.into(),
        port_on_a: PortId::default(),
        chan_on_a: ChannelId::default(),
        port_on_b: PortId::default(),
        chan_on_b: ChannelId::default(),
        data: Vec::new(),
        timeout_height_on_b: client_height.into(),
        timeout_timestamp_on_b: Timestamp::from_nanoseconds(1).unwrap(),
    };

    let msg_packet_old = MsgRecvPacket {
        packet: packet_old,
        proof_commitment_on_a: msg.proof_commitment_on_a.clone(),
        proof_height_on_a: msg.proof_height_on_a,
        signer: get_dummy_account_id(),
    };

    let msg = MsgEnvelope::Packet(PacketMsg::Recv(msg));
    let msg_packet_old = MsgEnvelope::Packet(PacketMsg::Recv(msg_packet_old));

    let chan_end_on_b = ChannelEnd::new(
        State::Open,
        Order::default(),
        Counterparty::new(packet.port_on_a, Some(packet.chan_on_a)),
        vec![ConnectionId::default()],
        Version::new("ics20-1".to_string()),
    );

    let conn_end_on_b = ConnectionEnd::new(
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
                    .clone()
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_b.clone())
                    .with_channel(
                        packet.port_on_b.clone(),
                        packet.chan_on_b.clone(),
                        chan_end_on_b.clone(),
                    )
                    .with_send_sequence(
                        packet.port_on_b.clone(),
                        packet.chan_on_b.clone(),
                        1.into(),
                    )
                    // .with_height(host_height)
                    // This `with_recv_sequence` is required for ordered channels
                    .with_recv_sequence(
                        packet.port_on_b.clone(),
                        packet.chan_on_b.clone(),
                        packet.sequence,
                    ),
            msg,
            want_pass: true,
        },
        Test {
            name: "Packet timeout expired".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_b)
                    .with_channel(PortId::default(), ChannelId::default(), chan_end_on_b)
                    .with_send_sequence(PortId::default(), ChannelId::default(), 1.into()),
                    // .with_height(host_height),
                msg: msg_packet_old,
                want_pass: false,
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
                        "recv_packet: test passed but was supposed to fail for test: {}, \nparams \n msg={:?}\nctx:{:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                assert!(!proto_output.events.is_empty()); // Some events must exist.

                for e in proto_output.events.iter() {
                    assert!(matches!(e, &IbcEvent::ReceivePacket(_)));
                }
            }
            Err(e) => {
                assert!(
                        !test.want_pass,
                        "recv_packet: did not pass test: {}, \nparams \nmsg={:?}\nctx={:?}\nerror={:?}",
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
