use crate::{Config, Event};
pub use alloc::{
	format,
	string::{String, ToString},
};
use codec::{Decode, Encode};
use ibc::core::host::types::identifiers::PortId;
use ibc::core::router::types::error::RouterError;
use sp_std::vec::Vec;

#[derive(PartialEq, Eq, Clone, Debug, scale_info::TypeInfo, Encode, Decode)]
pub enum IbcError {
	/// unknown type URL
	UnknownMessageTypeUrl { message: Vec<u8> },
	/// the message is malformed and cannot be decoded error: `{reason}`
	MalformedMessageBytes { reason: Vec<u8> },
	/// port `{port_id}` is unknown
	UnknownPort { port_id: PortId },
	/// module not found
	ModuleNotFound,
}

impl From<RouterError> for IbcError {
	fn from(err: RouterError) -> Self {
		match err {
			RouterError::UnknownMessageTypeUrl { url } => {
				IbcError::UnknownMessageTypeUrl { message: url.as_bytes().to_vec() }
			},
			RouterError::MalformedMessageBytes { reason } => {
				IbcError::MalformedMessageBytes { reason: reason.as_bytes().to_vec() }
			},
			RouterError::UnknownPort { port_id } => IbcError::UnknownPort { port_id },
			RouterError::ModuleNotFound => IbcError::ModuleNotFound,
		}
	}
}

impl<T: Config> From<Vec<RouterError>> for Event<T> {
	fn from(errors: Vec<RouterError>) -> Self {
		let errors: Vec<IbcError> = errors.into_iter().map(|err| err.into()).collect();
		Event::<T>::IbcErrors { errors }
	}
}
