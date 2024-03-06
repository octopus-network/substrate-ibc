// pub mod test_util {
// 	use ibc::core::client::types::Height;

// 	use ibc_proto::ibc::core::{
// 		client::v1::Height as RawHeight,
// 		connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck,
// 	};

// 	use crate::tests::common::{get_dummy_bech32_account, get_dummy_proof};
// 	use ibc::core::connection::types::version::Version;

// 	pub fn get_dummy_raw_msg_conn_open_ack(
// 		proof_height: u64,
// 		consensus_height: u64,
// 	) -> RawMsgConnectionOpenAck {
// 		let client_state_height = Height::new(0, consensus_height).unwrap();
// 		RawMsgConnectionOpenAck {
// 			connection_id: ConnectionId::new(0).to_string(),
// 			counterparty_connection_id: ConnectionId::new(1).to_string(),
// 			proof_try: get_dummy_proof(),
// 			proof_height: Some(RawHeight { revision_number: 0, revision_height: proof_height }),
// 			proof_consensus: get_dummy_proof(),
// 			consensus_height: Some(RawHeight {
// 				revision_number: 0,
// 				revision_height: consensus_height,
// 			}),
// 			client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
// 			proof_client: get_dummy_proof(),
// 			version: Some(Version::default().into()),
// 			signer: get_dummy_bech32_account(),
// 		}
// 	}
// }
