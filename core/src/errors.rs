pub use alloc::{
	format,
	string::{String, ToString},
};
use codec::{Decode, Encode};
use sp_std::vec::Vec;

#[derive(
	PartialEq, Eq, Clone, frame_support::RuntimeDebug, scale_info::TypeInfo, Encode, Decode,
)]
pub enum IbcError {
	/// context error
	ContextError { message: Vec<u8> },
	/// unknown type URL
	UnknownMessageTypeUrl { message: Vec<u8> },
	/// the message is malformed and cannot be decoded
	MalformedMessageBytes { message: Vec<u8> },
}
