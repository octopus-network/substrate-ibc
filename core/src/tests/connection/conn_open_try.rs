// pub mod test_util {
// 	use super::super::common::test_util::get_dummy_raw_counterparty;
// 	use crate::tests::common::{get_dummy_bech32_account, get_dummy_proof};
// 	use ibc::core::client::types::Height;
// 	use ibc::core::connection::types::version::get_compatible_versions;
// 	use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
// 	use ibc_proto::ibc::core::{
// 		client::v1::Height as RawHeight,
// 		connection::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry,
// 	};

// 	#[allow(deprecated)]
// 	/// Returns a dummy `RawMsgConnectionOpenTry` with parametrized heights. The parameter
// 	/// `proof_height` represents the height, on the source chain, at which this chain produced the
// 	/// proof. Parameter `consensus_height` represents the height of destination chain which a
// 	/// client on the source chain stores.
// 	pub fn get_dummy_raw_msg_conn_open_try(
// 		proof_height: u64,
// 		consensus_height: u64,
// 	) -> RawMsgConnectionOpenTry {
// 		let client_state_height = Height::new(0, consensus_height).unwrap();
// 		RawMsgConnectionOpenTry {
// 			client_id: ClientId::new(mock_client_type(), 0).unwrap().to_string(),
// 			previous_connection_id: ConnectionId::default().to_string(),
// 			client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
// 			counterparty: Some(get_dummy_raw_counterparty()),
// 			delay_period: 0,
// 			counterparty_versions: get_compatible_versions()
// 				.iter()
// 				.map(|v| v.clone().into())
// 				.collect(),
// 			proof_init: get_dummy_proof(),
// 			proof_height: Some(RawHeight { revision_number: 0, revision_height: proof_height }),
// 			proof_consensus: get_dummy_proof(),
// 			consensus_height: Some(RawHeight {
// 				revision_number: 0,
// 				revision_height: consensus_height,
// 			}),
// 			proof_client: get_dummy_proof(),
// 			signer: get_dummy_bech32_account(),
// 		}
// 	}
// }
