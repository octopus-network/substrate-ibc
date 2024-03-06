pub mod test_util {
	use ibc_proto::ibc::core::{
		channel::v1::MsgChannelCloseConfirm as RawMsgChannelCloseConfirm, client::v1::Height,
	};

	use crate::tests::common::{get_dummy_bech32_account, get_dummy_proof};
	use ibc::core::host::types::identifiers::{ChannelId, PortId};

	/// Returns a dummy `RawMsgChannelCloseConfirm`, for testing only!
	pub fn get_dummy_raw_msg_chan_close_confirm(proof_height: u64) -> RawMsgChannelCloseConfirm {
		RawMsgChannelCloseConfirm {
			port_id: PortId::transfer().to_string(),
			channel_id: ChannelId::default().to_string(),
			proof_init: get_dummy_proof(),
			proof_height: Some(Height { revision_number: 0, revision_height: proof_height }),
			signer: get_dummy_bech32_account(),
		}
	}
}
