#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
use sp_std::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait IbcApi {
		fn get_identified_any_client_state() -> Vec<(Vec<u8>, Vec<u8>)>;

		fn get_idenfitied_connection_end() -> Vec<(Vec<u8>, Vec<u8>)>;

		fn get_idenfitied_channel_end() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>;

		fn get_channel_end(port_id: Vec<u8>, channel_id: Vec<u8>) -> Vec<u8>;

		fn get_packet_commitment_state() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)>;

		fn get_packet_acknowledge_state() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)>;
	}
}