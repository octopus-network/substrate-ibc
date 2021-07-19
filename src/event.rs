use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

pub mod primitive {
    use ibc::ics02_client::height::Height as IbcHeight;
    use ibc::ics24_host::identifier::ClientId as IbcClientId;
    use ibc::ics02_client::client_type::ClientType as IbcClientType;

    use codec::{Decode, Encode};
    use sp_runtime::RuntimeDebug;

    #[derive(Clone, PartialEq, Eq,  Encode, Decode, RuntimeDebug)]
    pub struct Height {
        /// Previously known as "epoch"
        pub revision_number: u64,

        /// The height of a block
        pub revision_height: u64,
    }

    impl From<IbcHeight> for Height {
        fn from(IbcHeight{revision_number, revision_height} : IbcHeight) -> Self {
            Height {
                revision_number,
                revision_height,
            }
        }
    }

    #[derive(Clone, PartialEq, Eq,  Encode, Decode, RuntimeDebug)]
    pub enum ClientType {
        Tendermint,
    }

    impl From<IbcClientType> for ClientType {
        fn from(value: IbcClientType) -> Self {
            match value {
                IbcClientType::Tendermint => ClientType::Tendermint,
                _ => unreachable!()
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ClientId(String);

    impl From<IbcClientId> for ClientId {
        fn from(value: IbcClientId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }
}





