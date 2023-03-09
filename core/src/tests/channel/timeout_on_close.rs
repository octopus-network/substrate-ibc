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
