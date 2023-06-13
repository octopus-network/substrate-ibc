use crate::alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use ibc::{
	applications::transfer::{error::TokenTransferError, VERSION},
	core::{
		ics04_channel::timeout::TimeoutHeight,
		ics04_channel::{channel::Order, events::SendPacket as IbcSendPacket, packet::Sequence},
		ics24_host::identifier::{ChannelId as IbcChannelId, ConnectionId, PortId},
	},
	signer::Signer,
};
use scale_info::{prelude::format, TypeInfo};
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

/// Derive the transferred token denomination using
/// <https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md>
pub fn derive_ibc_denom_with_path(transfer_path: &str) -> Result<String, TokenTransferError> {
	use subtle_encoding::hex;
	let mut hasher = Sha256::new();
	hasher.update(transfer_path.as_bytes());

	let denom_bytes = hasher.finalize();
	let denom_hex = String::from_utf8(hex::encode_upper(denom_bytes))
		.map_err(|e| TokenTransferError::UnknownMsgType { msg_type: format!("error: {}", e) })?;

	let denom_str = format!("ibc/{}", denom_hex);
	log::info!(
		"🐙🐙 pallet_ics20_transfer::impls -> mint_coins denom_trace_hash: {:?} ",
		denom_str
	);
	Ok(denom_str)
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct SendPacket {
	packet_data: Vec<u8>,
	timeout_height: TimeoutHeight,
	timeout_timestamp: u64,
	sequence: Sequence,
	src_port_id: PortId,
	src_channel_id: IbcChannelId,
	dst_port_id: PortId,
	dst_channel_id: IbcChannelId,
	channel_ordering: Order,
	src_connection_id: ConnectionId,
}
impl From<IbcSendPacket> for SendPacket {
	fn from(v: IbcSendPacket) -> Self {
		Self {
			packet_data: (*v.packet_data()).to_vec(),
			timeout_height: *v.timeout_height(),
			timeout_timestamp: (*v.timeout_timestamp()).nanoseconds(),
			sequence: *v.sequence(),
			src_port_id: (*v.src_port_id()).to_owned(),
			src_channel_id: (*v.src_channel_id()).to_owned(),
			dst_port_id: (*v.dst_port_id()).to_owned(),
			dst_channel_id: (*v.dst_channel_id()).to_owned(),
			channel_ordering: (*v.channel_ordering()).to_owned(),
			src_connection_id: (*v.src_connection_id()).to_owned(),
		}
	}
}

#[test]
fn test_get_channel_escrow_address() {
	let signer = get_channel_escrow_address(&PortId::default(), &IbcChannelId::default()).unwrap();
	println!("{:?}", signer)
}
