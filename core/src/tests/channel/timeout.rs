pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::MsgTimeout as RawMsgTimeout, client::v1::Height as RawHeight,
	};

	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};

	/// Returns a dummy `RawMsgTimeout`, for testing only!
	/// The `height` parametrizes both the proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_timeout(
		proof_height: u64,
		timeout_height: u64,
		timeout_timestamp: u64,
	) -> RawMsgTimeout {
		RawMsgTimeout {
			packet: Some(get_dummy_raw_packet(timeout_height, timeout_timestamp)),
			proof_unreceived: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: proof_height }),
			next_sequence_recv: 1,
			signer: get_dummy_bech32_account(),
		}
	}
}
