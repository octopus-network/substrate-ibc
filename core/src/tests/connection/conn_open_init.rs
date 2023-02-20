pub mod test_util {
	use super::super::common::test_util::get_dummy_raw_counterparty;
	use crate::tests::common::get_dummy_bech32_account;
	use ibc::{
		core::{ics03_connection::version::Version, ics24_host::identifier::ClientId},
		mock::client_state::client_type as mock_client_type,
	};
	use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit as RawMsgConnectionOpenInit;

	#[allow(dead_code)]
	/// Returns a dummy message, for testing only.
	/// Other unit tests may import this if they depend on a MsgConnectionOpenInit.
	pub fn get_dummy_raw_msg_conn_open_init() -> RawMsgConnectionOpenInit {
		RawMsgConnectionOpenInit {
			client_id: ClientId::new(mock_client_type(), 0).unwrap().to_string(),
			counterparty: Some(get_dummy_raw_counterparty()),
			version: Some(Version::default().into()),
			delay_period: 0,
			signer: get_dummy_bech32_account(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::test_util::get_dummy_raw_msg_conn_open_init;
	use crate::{
		mock::{new_test_ext, Test as PalletIbcTest},
		Context,
	};
	use ibc::core::{
		ics03_connection::context::ConnectionReader,
		ics26_routing::{handler::dispatch, msgs::MsgEnvelope},
	};

	use ibc::{
		core::ics03_connection::{
			msgs::{conn_open_init::MsgConnectionOpenInit, ConnectionMsg},
			version::Version,
		},
		Height,
	};

	use ibc_proto::ibc::core::connection::v1::Version as RawVersion;

	#[test]
	fn conn_open_init_msg_processing() {
		#[allow(dead_code)]
		struct Test {
			name: String,
			ctx: Context<PalletIbcTest>,
			msg: MsgEnvelope,
			expected_versions: Vec<Version>,
			want_pass: bool,
		}

		let msg_conn_init_default =
			MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();
		let msg_conn_init_no_version =
			MsgConnectionOpenInit { version: None, ..msg_conn_init_default.clone() };
		let msg_conn_init_bad_version = MsgConnectionOpenInit {
			version: Version::try_from(RawVersion {
				identifier: "random identifier 424242".to_string(),
				features: vec![],
			})
			.unwrap()
			.into(),
			..msg_conn_init_default.clone()
		};
		new_test_ext().execute_with(|| {
			let default_context = Context::<PalletIbcTest>::new();
			let good_context = default_context
				.clone()
				.with_client(&msg_conn_init_default.client_id_on_a, Height::new(0, 10).unwrap());

			let tests: Vec<Test> = vec![
				// todo
				// Test {
				// 	name: "Processing fails because no client exists in the context".to_string(),
				// 	ctx: default_context,
				// 	msg: MsgEnvelope::Connection(ConnectionMsg::OpenInit(msg_conn_init_default.
				// clone())), 	expected_versions:
				// vec![msg_conn_init_default.version.clone().unwrap()], 	want_pass: false,
				// },
				Test {
					name: "Incompatible version in MsgConnectionOpenInit msg".to_string(),
					ctx: good_context.clone(),
					msg: MsgEnvelope::Connection(ConnectionMsg::OpenInit(
						msg_conn_init_bad_version,
					)),
					expected_versions: vec![],
					want_pass: false,
				},
				Test {
					name: "No version in MsgConnectionOpenInit msg".to_string(),
					ctx: good_context.clone(),
					msg: MsgEnvelope::Connection(ConnectionMsg::OpenInit(msg_conn_init_no_version)),
					expected_versions: good_context.get_compatible_versions(),
					want_pass: true,
				},
				Test {
					name: "Good parameters".to_string(),
					ctx: good_context,
					msg: MsgEnvelope::Connection(ConnectionMsg::OpenInit(
						msg_conn_init_default.clone(),
					)),
					expected_versions: vec![msg_conn_init_default.version.unwrap()],
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
						assert!(!proto_output.events.is_empty()); // Some events must exist.

						// The object in the output is a ConnectionEnd, should have init state.
						// 	let res: ConnectionResult = proto_output.result;
						// 	assert_eq!(res.connection_end.state().clone(), State::Init);

						// 	for e in proto_output.events.iter() {
						// 		assert!(matches!(e, &IbcEvent::OpenInitConnection(_)));
						// 	}

						// 	assert_eq!(res.connection_end.versions(), test.expected_versions);

						// 	// This needs to be last
						// 	assert!(
						// 	test.want_pass,
						// 	"conn_open_init: test passed but was supposed to fail for test: {},
						// \nparams {:?} {:?}", 	test.name,
						// 	test.msg.clone(),
						// 	test.ctx.clone()
						// );
					},
					Err(e) => {
						assert!(
							!test.want_pass,
							"conn_open_init: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
							test.name,
							test.msg,
							test.ctx.clone(),
							e,
						);
					},
				}
			}
		})
	}
}
