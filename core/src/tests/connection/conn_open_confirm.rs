pub mod test_util {
	use ibc_proto::ibc::core::{
		client::v1::Height, connection::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm,
	};

	use crate::tests::common::{get_dummy_bech32_account, get_dummy_proof};

	pub fn get_dummy_raw_msg_conn_open_confirm() -> RawMsgConnectionOpenConfirm {
		RawMsgConnectionOpenConfirm {
			connection_id: "srcconnection".to_string(),
			proof_ack: get_dummy_proof(),
			proof_height: Some(Height { revision_number: 0, revision_height: 10 }),
			signer: get_dummy_bech32_account(),
		}
	}
}
