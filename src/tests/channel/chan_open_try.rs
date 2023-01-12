pub mod test_util {

	use ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;

	use crate::tests::{
		channel::common::test_util::get_dummy_raw_channel_end,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};
	use ibc::core::ics24_host::identifier::{ChannelId, PortId};

	use ibc_proto::ibc::core::client::v1::Height;

	/// Returns a dummy `RawMsgChannelOpenTry`, for testing only!
	pub fn get_dummy_raw_msg_chan_open_try(proof_height: u64) -> RawMsgChannelOpenTry {
		#[allow(deprecated)]
		RawMsgChannelOpenTry {
			port_id: PortId::transfer().to_string(),
			previous_channel_id: ChannelId::default().to_string(),
			channel: Some(get_dummy_raw_channel_end()),
			counterparty_version: "ics20-1".to_string(),
			proof_init: get_dummy_proof(),
			proof_height: Some(Height { revision_number: 0, revision_height: proof_height }),
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::test_util::get_dummy_raw_msg_chan_open_try;
	use crate::{
		mock::{new_test_ext, Test as PalletIbcTest},
		tests::connection::common::test_util::get_dummy_raw_counterparty,
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
				channel::{ChannelEnd, Counterparty, State},
				error::ChannelError,
				msgs::{chan_open_try::MsgChannelOpenTry, ChannelMsg},
				Version,
			},
			ics24_host::identifier::{ChannelId, ClientId, ConnectionId},
			ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
		},
		mock::client_state::client_type as mock_client_type,
		timestamp::ZERO_DURATION,
		Height,
	};

	#[test]
	#[ignore]
	fn chan_open_try_msg_processing() {
		new_test_ext().execute_with(|| {
    #[allow(dead_code)]
    struct Test {
        name: String,
        ctx: Context<PalletIbcTest>,
        msg: MsgEnvelope,
        want_pass: bool,
        match_error: Box<dyn FnOnce(ChannelError)>,
    }

    // Some general-purpose variable to parametrize the messages and the context.
    let proof_height = 10;
    let conn_id = ConnectionId::new(2);
    let client_id = ClientId::new(mock_client_type(), 45).unwrap();

    // The context. We'll reuse this same one across all tests.
    let context = Context::<PalletIbcTest>::new();

    // This is the connection underlying the channel we're trying to open.
    let conn_end = ConnectionEnd::new(
        ConnectionState::Open,
        client_id.clone(),
        ConnectionCounterparty::try_from(get_dummy_raw_counterparty()).unwrap(),
        get_compatible_versions(),
        ZERO_DURATION,
    );

    // We're going to test message processing against this message.
    let mut msg =
        MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

    let chan_id = ChannelId::new(24);
    let hops = vec![conn_id.clone()];
    msg.connection_hops_on_b = hops;

    // A preloaded channel end that resides in the context. This is constructed so as to be
    // consistent with the incoming ChanOpenTry message `msg`.
    let correct_chan_end = ChannelEnd::new(
        State::Init,
        msg.ordering,
        Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
        msg.connection_hops_on_b.clone(),
        Version::empty(),
    );

    let tests: Vec<Test> = vec![
        // todo
        // Test {
        //     name: "Processing fails because no connection exists in the context".to_string(),
        //     ctx: context.clone(),
        //     msg: MsgEnvelope::Channel(ChannelMsg::OpenTry(msg.clone())),
        //     want_pass: false,
        //     match_error: {
        //         let connection_id = msg.chan_end_on_b.connection_hops()[0].clone();
        //         Box::new(move |e| match e {
        //             error::ErrorDetail::Ics03Connection(e) => {
        //                 assert_eq!(
        //                     e.source,
        //                     ics03_error::ErrorDetail::ConnectionNotFound(
        //                         ics03_error::ConnectionNotFoundSubdetail { connection_id }
        //                     )
        //                 );
        //             }
        //             _ => {
        //                 panic!("Expected MissingConnection, instead got {}", e)
        //             }
        //         })
        //     },
        // },
        // Test {
        //     name: "Processing fails b/c the context has no client state".to_string(),
        //     ctx: context
        //         .clone()
        //         .with_connection(conn_id.clone(), conn_end.clone())
        //         .with_channel(
        //             msg.port_id_on_b.clone(),
        //             chan_id.clone(),
        //             correct_chan_end.clone(),
        //         ),
        //     msg: MsgEnvelope::Channel(ChannelMsg::OpenTry(msg.clone())),
        //     want_pass: false,
        //     match_error: Box::new(|e| match e {
        //         error::ErrorDetail::Ics03Connection(e) => {
        //             assert_eq!(
        //                 e.source,
        //                 ics03_error::ErrorDetail::Ics02Client(
        //                     ics03_error::Ics02ClientSubdetail {
        //                         source: ics02_error::ErrorDetail::ClientNotFound(
        //                             ics02_error::ClientNotFoundSubdetail {
        //                                 client_id: ClientId::new(mock_client_type(), 45)
        //                                     .unwrap()
        //                             }
        //                         )
        //                     }
        //                 )
        //             );
        //         }
        //         _ => {
        //             panic!("Expected MissingClientState, instead got {}", e)
        //         }
        //     }),
        // },
        Test {
            name: "Processing is successful".to_string(),
            ctx: context
                .clone()
                .with_client(&client_id, Height::new(0, proof_height).unwrap())
                .with_connection(conn_id.clone(), conn_end.clone())
                .with_channel(msg.port_id_on_b.clone(), chan_id, correct_chan_end),
            msg: MsgEnvelope::Channel(ChannelMsg::OpenTry(msg.clone())),
            want_pass: true,
            match_error: Box::new(|_| {}),
        },
        Test {
            name: "Processing is successful against an empty context (no preexisting channel)"
                .to_string(),
            ctx: context
                .with_client(&client_id, Height::new(0, proof_height).unwrap())
                .with_connection(conn_id, conn_end),
            msg: MsgEnvelope::Channel(ChannelMsg::OpenTry(msg)),
            want_pass: true,
            match_error: Box::new(|_| {}),
        },
    ]
    .into_iter()
    .collect();

    for test in tests {
        let mut test = test;
        let test_msg = test.msg;
        let res = dispatch(&mut test.ctx, test_msg.clone());
        // Additionally check the events and the output objects in the result.
        match res {
            Ok(_res) => {
                assert!(
                    test.want_pass,
                    "chan_open_ack: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                    test.name,
                    test_msg,
                    test.ctx.clone()
                );
                // The object in the output is a channel end, should have TryOpen state.
                // assert_eq!(res.channel_end.state().clone(), State::TryOpen);
            }
            Err(e) => {
                assert!(
                    !test.want_pass,
                    "chan_open_try: did not pass test: {}, \nparams:\n\tmsg={:?}\n\tcontext={:?}\nerror: {:?}",
                    test.name,
                    test_msg,
                    test.ctx.clone(),
                    e,
                );

                // (test.match_error)(e);
            }
        }
    }
})
	}

	/// Addresses [issue 219](https://github.com/cosmos/ibc-rs/issues/219)
	#[test]
	fn chan_open_try_invalid_counterparty_channel_id() {
		new_test_ext().execute_with(|| {
			let proof_height = 10;
			let conn_id = ConnectionId::new(2);
			let client_id = ClientId::new(mock_client_type(), 45).unwrap();

			// This is the connection underlying the channel we're trying to open.
			let conn_end = ConnectionEnd::new(
				ConnectionState::Open,
				client_id.clone(),
				ConnectionCounterparty::try_from(get_dummy_raw_counterparty()).unwrap(),
				get_compatible_versions(),
				ZERO_DURATION,
			);

			// We're going to test message processing against this message.
			// Note: we make the counterparty's channel_id `None`.
			let mut msg =
				MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

			let chan_id = ChannelId::new(24);
			let hops = vec![conn_id.clone()];
			msg.connection_hops_on_b = hops;

			let chan_end = ChannelEnd::new(
				State::Init,
				msg.ordering,
				Counterparty::new(msg.port_id_on_a.clone(), None),
				msg.connection_hops_on_b.clone(),
				Version::empty(),
			);

			let mut context = Context::<PalletIbcTest>::new()
				.with_client(&client_id, Height::new(0, proof_height).unwrap())
				.with_connection(conn_id, conn_end)
				.with_channel(msg.port_id_on_b.clone(), chan_id, chan_end);

			// Makes sure we don't crash
			let _ = dispatch(&mut context, MsgEnvelope::Channel(ChannelMsg::OpenTry(msg)));
		})
	}
}
