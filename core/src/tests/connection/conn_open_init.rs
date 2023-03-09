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
