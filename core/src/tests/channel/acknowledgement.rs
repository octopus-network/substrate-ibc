pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::{MsgAcknowledgement as RawMsgAcknowledgement, Packet as RawPacket},
		client::v1::Height as RawHeight,
	};

	use crate::tests::{
		channel::packet::test_utils::get_dummy_raw_packet,
		common::{get_dummy_bech32_account, get_dummy_proof},
	};

	/// Returns a dummy `RawMsgAcknowledgement`, for testing only!
	/// The `height` parametrizes both the proof height as well as the timeout height.
	pub fn get_dummy_raw_msg_acknowledgement(height: u64) -> RawMsgAcknowledgement {
		get_dummy_raw_msg_ack_with_packet(get_dummy_raw_packet(height, 1), height)
	}

	pub fn acknowledgement() -> Vec<u8> {
		use ibc::apps::transfer::types::ACK_SUCCESS_B64;
		ACK_SUCCESS_B64.as_bytes().to_vec()
	}

	pub fn get_dummy_raw_msg_ack_with_packet(
		packet: RawPacket,
		height: u64,
	) -> RawMsgAcknowledgement {
		RawMsgAcknowledgement {
			packet: Some(packet),
			acknowledgement: acknowledgement(),
			proof_acked: get_dummy_proof(),
			proof_height: Some(RawHeight { revision_number: 0, revision_height: height }),
			signer: get_dummy_bech32_account(),
		}
	}
}
