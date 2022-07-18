use crate::Config;
use codec::Encode;
use scale_info::prelude::{fmt::Debug, format, vec::Vec};

use ibc::{
	applications::transfer::{error::Error as Ics20Error, VERSION},
	core::ics24_host::identifier::{ChannelId, PortId},
	signer::Signer,
};


pub fn get_channel_escrow_address(
	port_id: &PortId,
	channel_id: &ChannelId,
) -> Result<Signer, Ics20Error> {
	let contents = format!("{}/{}", port_id, channel_id);
	let mut data = VERSION.as_bytes().to_vec();
	data.extend_from_slice(&[0]);
	data.extend_from_slice(contents.as_bytes());

	let hash = sp_io::hashing::sha2_256(&data).to_vec();
	let mut hex_string = hex::encode_upper(hash);
	hex_string.insert_str(0, "0x");
	hex_string.parse::<Signer>().map_err(Ics20Error::signer)
}