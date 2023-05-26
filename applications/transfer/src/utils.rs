use alloc::string::String;
use ibc::{
	applications::transfer::{error::TokenTransferError, VERSION},
	core::ics24_host::identifier::{ChannelId as IbcChannelId, PortId},
	signer::Signer,
};
use scale_info::prelude::format;
use sha2::{Digest, Sha256};

/// In ICS20 fungible token transfer, get the escrow address by channel ID and port ID
///
/// Parameters:
/// - `port_id`: The ID of the port corresponding to the escrow.
/// - `channel_id`: The ID of the channel corresponding to the escrow.
pub fn get_channel_escrow_address(
	port_id: &PortId,
	channel_id: &IbcChannelId,
) -> Result<Signer, TokenTransferError> {
	let contents = format!("{}/{}", port_id, channel_id);
	let mut data = VERSION.as_bytes().to_vec();
	data.extend_from_slice(&[0]);
	data.extend_from_slice(contents.as_bytes());

	let hash = sp_io::hashing::sha2_256(&data).to_vec();
	let mut hex_string = hex::encode_upper(hash);
	hex_string.insert_str(0, "0x");
	hex_string.parse::<Signer>().map_err(TokenTransferError::Signer)
}

// pub fn derive_ibc_denom(
// 	port_id: &PortId,
// 	channel_id: &IbcChannelId,
// 	denom: &str,
// ) -> Result<String, TokenTransferError> {
// 	let transfer_path = format!("{}/{}/{}", port_id, channel_id, denom);
// 	derive_ibc_denom_with_path(&transfer_path)
// }

/// Derive the transferred token denomination using
/// <https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md>
pub fn derive_ibc_denom_with_path(transfer_path: &str) -> Result<String, TokenTransferError> {
	use subtle_encoding::hex;
	let mut hasher = Sha256::new();
	hasher.update(transfer_path.as_bytes());

	let denom_bytes = hasher.finalize();
	let denom_hex = String::from_utf8(hex::encode_upper(denom_bytes))
		.map_err(|e| TokenTransferError::UnknownMsgType { msg_type: format!("error: {}", e) })?;

	Ok(format!("ibc/{}", denom_hex))
}

#[test]
fn test_get_channel_escrow_address() {
	let signer = get_channel_escrow_address(&PortId::default(), &IbcChannelId::default()).unwrap();
	println!("{:?}", signer)
}
