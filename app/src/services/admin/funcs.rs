use crate::admin::{
    storage::AdditionalMeta,
    utils::{Error, Result},
};
use crate::services::erc20::{
    funcs,
    utils::{AllowancesMap, BalancesMap, NonZeroU256},
};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub fn mint(
    balances: &mut BalancesMap,
    meta: &AdditionalMeta,
    total_supply: &mut U256,
    to: ActorId,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }

    let new_total_supply = total_supply
        .checked_add(value)
        .ok_or(Error::NumericOverflow)?;
    if new_total_supply <= meta.max_supply {
        let new_to = funcs::balance_of(balances, to)
            .checked_add(value)
            .ok_or(Error::NumericOverflow)?;

        let Ok(non_zero_new_to) = new_to.try_into() else {
            unreachable!("Infallible since fn is noop on zero value; qed");
        };

        balances.insert(to, non_zero_new_to);
    } else {
        return Err(Error::MaxSupplyReached);
    }
    *total_supply = new_total_supply;
    Ok(true)
}

pub fn burn(
    balances: &mut BalancesMap,
    total_supply: &mut U256,
    from: ActorId,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }
    let new_total_supply = total_supply.checked_sub(value).ok_or(Error::Underflow)?;

    let new_from = funcs::balance_of(balances, from)
        .checked_sub(value)
        .ok_or(Error::InsufficientBalance)?;

    if let Ok(non_zero_new_from) = new_from.try_into() {
        balances.insert(from, non_zero_new_from);
    } else {
        balances.remove(&from);
    }
    *total_supply = new_total_supply;

    Ok(true)
}

pub fn transfer_to_users(
    balances: &mut BalancesMap,
    from: ActorId,
    to: Vec<ActorId>,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }

    let new_from = funcs::balance_of(balances, from)
        .checked_sub(value * to.len())
        .ok_or(Error::InsufficientBalance)?;

    for user_id in to.clone() {
        let new_to = funcs::balance_of(balances, user_id)
            .checked_add(value)
            .ok_or(Error::NumericOverflow)?;

        let Ok(non_zero_new_to) = new_to.try_into() else {
            unreachable!("Infallible since fn is noop on zero value; qed");
        };

        balances.insert(user_id, non_zero_new_to);
    }

    if let Ok(non_zero_new_from) = new_from.try_into() {
        balances.insert(from, non_zero_new_from);
    } else {
        balances.remove(&from);
    }

    Ok(true)
}

pub fn maps_data(
    allowances: &AllowancesMap,
    balances: &BalancesMap,
) -> ((usize, usize), (usize, usize)) {
    (
        (allowances.len(), allowances.capacity()),
        (balances.len(), balances.capacity()),
    )
}

pub fn allowances_reserve(allowances: &mut AllowancesMap, additional: usize) {
    allowances.reserve(additional)
}

pub fn balances_reserve(balances: &mut BalancesMap, additional: usize) {
    balances.reserve(additional)
}

pub fn allowances(
    allowances: &AllowancesMap,
    skip: usize,
    take: usize,
) -> Vec<((ActorId, ActorId), NonZeroU256)> {
    allowances
        .iter()
        .skip(skip)
        .take(take)
        .map(|(&(id1, id2), &v)| ((id1, id2), v))
        .collect()
}

pub fn balances(balances: &BalancesMap, skip: usize, take: usize) -> Vec<(ActorId, NonZeroU256)> {
    balances
        .iter()
        .skip(skip)
        .take(take)
        .map(|(&id, &v)| (id, v))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::funcs;
    use utils::*;

    macro_rules! assert_ok {
        ( $x:expr, $y: expr $(,)? ) => {{
            assert_eq!($x.unwrap(), $y);
        }};
    }

    macro_rules! assert_err {
        ( $x:expr, $y: expr $(,)? ) => {{
            assert_eq!($x.err().expect("Ran into Ok value"), $y);
        }};
    }

    #[test]
    fn mint() {
        // Initializing thread logger.
        let _ = env_logger::try_init();

        // Creating map
        let mut map = balances_map([]);
        let additional_meta = additional_meta();
        let mut total_supply: U256 = 100.into();
        let value: U256 = 100.into();
        // # Test case #1.
        // Successful Mint
        {
            assert_ok!(
                funcs::mint(
                    &mut map,
                    &additional_meta,
                    &mut total_supply,
                    alice(),
                    value
                ),
                true
            );
            let expected_balance: NonZeroU256 = value.try_into().unwrap();
            assert_eq!(*map.get(&alice()).unwrap(), expected_balance);
            assert_eq!(total_supply, 200.into());
        }
        // # Test case #2.
        // Mint with value equal to 0
        {
            assert_ok!(
                funcs::mint(
                    &mut map,
                    &additional_meta,
                    &mut total_supply,
                    alice(),
                    0.into()
                ),
                false
            );
        }

        // # Test case #3.
        // Mint with Error: MaxSupplyReached
        {
            assert_err!(
                funcs::mint(
                    &mut map,
                    &additional_meta,
                    &mut total_supply,
                    alice(),
                    1_000_000.into()
                ),
                Error::MaxSupplyReached
            );
        }
    }

    #[test]
    fn burn() {
        // Initializing thread logger.
        let _ = env_logger::try_init();

        // Creating map
        let mut map = balances_map([(dave(), U256::MAX)]);
        let mut total_supply: U256 = 100.into();
        let value: U256 = 100.into();
        // # Test case #1.
        // Successful Burn
        {
            assert_ok!(
                funcs::burn(&mut map, &mut total_supply, dave(), value),
                true
            );
            let expected_balance: NonZeroU256 = (U256::MAX - value).try_into().unwrap();
            assert_eq!(*map.get(&dave()).unwrap(), expected_balance);
            assert_eq!(total_supply, 0.into());
        }
        // # Test case #2.
        // Burn with value equal to 0
        {
            assert_ok!(
                funcs::burn(&mut map, &mut total_supply, dave(), 0.into()),
                false
            );
        }

        // # Test case #3.
        // Burn with Error: Underflow
        {
            assert_err!(
                funcs::burn(&mut map, &mut total_supply, alice(), value),
                Error::Underflow
            );
        }
    }

    #[test]
    fn transfer_to_users() {
        // Initializing thread logger.
        let _ = env_logger::try_init();

        // Creating map
        let mut map = balances_map([(dave(), U256::MAX)]);
        let value: U256 = 100.into();
        let to = vec![alice(), bob(), charlie()];
        let to_len: U256 = to.len().into();
        // # Test case #1.
        // Successful Transfer to users
        {
            assert_ok!(
                funcs::transfer_to_users(&mut map, dave(), to.clone(), value),
                true
            );
            let expected_balance: NonZeroU256 = (U256::MAX - to_len * value).try_into().unwrap();
            assert_eq!(*map.get(&dave()).unwrap(), expected_balance);
            let expected_balance: NonZeroU256 = value.try_into().unwrap();
            assert_eq!(*map.get(&alice()).unwrap(), expected_balance);
            assert_eq!(*map.get(&bob()).unwrap(), expected_balance);
            assert_eq!(*map.get(&charlie()).unwrap(), expected_balance);
        }
        // # Test case #2.
        // Burn with Error: InsufficientBalance
        {
            assert_err!(
                funcs::transfer_to_users(&mut map, alice(), to, value),
                Error::InsufficientBalance
            );
        }
    }

    mod utils {
        use super::{AdditionalMeta, AllowancesMap, BalancesMap};
        use crate::admin::ExternalLinks;
        use gstd::{ActorId, ToString};
        use primitive_types::U256;

        pub fn allowances_map<const N: usize>(
            content: [(ActorId, ActorId, U256); N],
        ) -> AllowancesMap {
            content
                .into_iter()
                .map(|(k1, k2, v)| ((k1, k2), v.try_into().unwrap()))
                .collect()
        }

        pub fn additional_meta() -> AdditionalMeta {
            AdditionalMeta {
                description: "Description".to_string(),
                external_links: ExternalLinks {
                    image: "image".to_string(),
                    website: None,
                    telegram: None,
                    twitter: None,
                    discord: None,
                    tokenomics: None,
                },
                max_supply: 1_000.into(),
            }
        }

        pub fn balances_map<const N: usize>(content: [(ActorId, U256); N]) -> BalancesMap {
            content
                .into_iter()
                .map(|(k, v)| (k, v.try_into().unwrap()))
                .collect()
        }

        pub fn alice() -> ActorId {
            1u64.into()
        }

        pub fn bob() -> ActorId {
            2u64.into()
        }

        pub fn charlie() -> ActorId {
            3u64.into()
        }

        pub fn dave() -> ActorId {
            4u64.into()
        }
    }
}
