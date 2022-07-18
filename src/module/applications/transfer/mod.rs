pub mod transfer_handle_callback;
use frame_support::traits::{
    fungibles::{Mutate, Transfer},
    ExistenceRequirement::AllowDeath,
};
use crate::{context::Context, *};
use log::{trace, error};

use ibc::{
    applications::transfer::{
        context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader},
        error::Error as Ics20Error,
        PrefixedCoin,
    },
    core::ics24_host::identifier::PortId,
    signer::Signer,
};
use ibc::applications::transfer::PORT_ID_STR;
use sp_runtime::{
    traits::{CheckedConversion, IdentifyAccount, Verify},
    MultiSignature,
};
use crate::utils::get_channel_escrow_address;

impl<T: Config> Ics20Keeper for Context<T> {
    type AccountId = <Self as Ics20Context>::AccountId;
}

impl<T: Config> BankKeeper for Context<T> {
    type AccountId = <Self as Ics20Context>::AccountId;

    fn send_coins(
        &mut self,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        let is_native_asset = amt.denom.trace_path().is_empty();
        match is_native_asset {
            // transfer native token
            true => {
                let amount = amt.amount.as_u256().low_u128().checked_into().unwrap(); // TODO: FIX IN THE FUTURE
                let native_token_name = T::NATIVE_TOKEN_NAME;
                let ibc_token_name = amt.denom.base_denom().as_str().as_bytes();

                // assert native token name equal want to send ibc token name
                assert_eq!(
                    native_token_name, ibc_token_name,
                    "send ibc token name is not native token name"
                );

                <T::Currency as Currency<T::AccountId>>::transfer(
                    &from.clone().into_account(),
                    &to.clone().into_account(),
                    amount,
                    AllowDeath,
                )
                    .map_err(|error| {
                        error!("❎ [send_coins] : Error: ({:?})", error);
                        Ics20Error::invalid_token()
                    })?;
            },
            // transfer non-native token
            false => {
                let amount = amt.amount.as_u256().low_u128().into();
                let denom = amt.denom.base_denom().as_str();
                // look cross chain asset have register in host chain
                match T::AssetIdByName::try_get_asset_id(denom) {
                    Ok(token_id) => {
                        <T::Assets as Transfer<T::AccountId>>::transfer(
                            token_id.into(),
                            &from.clone().into_account(),
                            &to.clone().into_account(),
                            amount,
                            true,
                        )
                            .map_err(|error| {
                                error!("❎ [send_coins] : Error: ({:?})", error);
                                Ics20Error::invalid_token()
                            })?;
                    },
                    Err(_error) => {
                        error!("❎ [send_coins]: denom: ({:?})", denom);
                        return Err(Ics20Error::invalid_token())
                    },
                }
            },
        }

        Ok(())
    }

    fn mint_coins(
        &mut self,
        account: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        let amount = amt.amount.as_u256().low_u128().into();
        let denom = amt.denom.base_denom().as_str();
        // look cross chain asset have register in host chain
        match T::AssetIdByName::try_get_asset_id(denom) {
            Ok(token_id) => {
                <T::Assets as Mutate<T::AccountId>>::mint_into(
                    token_id.into(),
                    &account.clone().into_account(),
                    amount,
                )
                    .map_err(|error| {
                        error!("❎ [mint_coins] : Error: ({:?})", error);
                        Ics20Error::invalid_token()
                    })?;
            },
            Err(_error) => {
                error!("❎ [mint_coins]: denom: ({:?})", denom);
                return Err(Ics20Error::invalid_token())
            },
        }
        Ok(())
    }

    fn burn_coins(
        &mut self,
        account: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        let amount = amt.amount.as_u256().low_u128().into();
        let denom = amt.denom.base_denom().as_str();
        // look cross chain asset have register in host chain
        match T::AssetIdByName::try_get_asset_id(denom) {
            Ok(token_id) => {
                <T::Assets as Mutate<T::AccountId>>::burn_from(
                    token_id.into(),
                    &account.clone().into_account(),
                    amount,
                )
                    .map_err(|error| {
                        error!("❎ [burn_coins] : Error: ({:?})", error);
                        Ics20Error::invalid_token()
                    })?;
            },
            Err(_error) => {
                error!("❎ [burn_coins]: denom: ({:?})", denom);
                return Err(Ics20Error::invalid_token())
            },
        }
        Ok(())
    }
}

impl<T: Config> Ics20Reader for Context<T> {
    type AccountId = <Self as Ics20Context>::AccountId;

    fn get_port(&self) -> Result<PortId, Ics20Error> {
        PortId::from_str(PORT_ID_STR)
            .map_err(|e| Ics20Error::invalid_port_id(PORT_ID_STR.to_string(), e))
    }

    fn get_channel_escrow_address(
        &self,
        port_id: &PortId,
        channel_id: &IbcChannelId,
    ) -> Result<Self::AccountId, Ics20Error> {
        get_channel_escrow_address(port_id, channel_id)?
            .try_into()
            .map_err(|_| Ics20Error::parse_account_failure())
    }

    fn is_send_enabled(&self) -> bool {
        // TODO(davirain), need according channelEnd def
        true
    }

    fn is_receive_enabled(&self) -> bool {
        // TODO(davirain), need according channelEnd def
        true
    }
}

impl<T: Config> Ics20Context for Context<T> {
    type AccountId = <T as Config>::AccountIdConversion; // Need Setting Account TODO(davirian)
}


// this is just mock, when at add this pallet to runtime need to config
// TODO IBC Account ID
type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone)]
pub struct IbcAccount(AccountId);

impl IdentifyAccount for IbcAccount {
    type AccountId = AccountId;
    fn into_account(self) -> Self::AccountId {
        self.0
    }
}

impl TryFrom<Signer> for IbcAccount {
    type Error = &'static str;
    // TODO
    fn try_from(_: Signer) -> Result<Self, Self::Error> {
        Ok(IbcAccount(AccountId::new([0u8; 32])))
    }
}
