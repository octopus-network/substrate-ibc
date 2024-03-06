pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::MsgRecvPacket as RawMsgRecvPacket, client::v1::Height as RawHeight,
	};

	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};
	use core::{ops::Add, time::Duration};
	use ibc::core::primitives::Timestamp;

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
