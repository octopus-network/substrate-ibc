pub mod test_util {
	use ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck as RawMsgChannelOpenAck;

	use crate::tests::common::{get_dummy_bech32_account, get_dummy_proof};
	use ibc::core::host::types::identifiers::{ChannelId, PortId};
	use ibc_proto::ibc::core::client::v1::Height;

	/// Returns a dummy `RawMsgChannelOpenAck`, for testing only!
	pub fn get_dummy_raw_msg_chan_open_ack(proof_height: u64) -> RawMsgChannelOpenAck {
		RawMsgChannelOpenAck {
			port_id: PortId::transfer().to_string(),
			channel_id: ChannelId::default().to_string(),
			counterparty_channel_id: ChannelId::default().to_string(),
			counterparty_version: "ics20-1".to_string(),
			proof_try: get_dummy_proof(),
			proof_height: Some(Height { revision_number: 0, revision_height: proof_height }),
			signer: get_dummy_bech32_account(),
		}
	}
}
