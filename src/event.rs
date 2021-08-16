use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

pub mod primitive {
    use ibc::ics02_client::client_type::ClientType as IbcClientType;
    use ibc::ics02_client::height::Height as IbcHeight;
    use ibc::ics24_host::identifier::ClientId as IbcClientId;
    use ibc::ics24_host::identifier::ConnectionId as IbcConnectionId;

    use codec::{Decode, Encode};
    use sp_runtime::RuntimeDebug;

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct Height {
        /// Previously known as "epoch"
        pub revision_number: u64,

        /// The height of a block
        pub revision_height: u64,
    }

    impl From<IbcHeight> for Height {
        fn from(
            IbcHeight {
                revision_number,
                revision_height,
            }: IbcHeight,
        ) -> Self {
            Height {
                revision_number,
                revision_height,
            }
        }
    }

    impl From<Height> for IbcHeight {
        fn from(val : Height) -> Self {
            Self {
                revision_number: val.revision_number,
                revision_height: val.revision_height,
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub enum ClientType {
        Tendermint,
        Grandpa,
    }

    impl From<IbcClientType> for ClientType {
        fn from(value: IbcClientType) -> Self {
            match value {
                IbcClientType::Tendermint => ClientType::Tendermint,
                IbcClientType::Grandpa => ClientType::Grandpa,
                _ => unreachable!(),
            }
        }
    }

    impl From<ClientType> for IbcClientType {
        fn from(val: ClientType) -> Self {
            match val {
                ClientType::Tendermint => IbcClientType::Tendermint,
                ClientType::Grandpa => IbcClientType::Grandpa,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ClientId(pub String);

    impl ClientId {
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcClientId> for ClientId {
        fn from(value: IbcClientId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl From<ClientId> for IbcClientId {
        fn from(val : ClientId) -> Self {
            let val = val.as_str();
            Self(val.to_string())
        }
    }

    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
    pub struct ConnectionId(String);

    impl ConnectionId {
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<IbcConnectionId> for ConnectionId {
        fn from(value: IbcConnectionId) -> Self {
            let value = value.as_str();
            Self(value.to_string())
        }
    }

    impl From<ConnectionId> for IbcConnectionId {
        fn from(val : ConnectionId) -> Self {
            let val = val.as_str();
            Self(val.to_string())
        }
    }
}
