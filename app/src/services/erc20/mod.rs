#![allow(clippy::unused_unit)]

mod internal;
mod storage;

pub use internal::*;

use self::storage::{
    allowances::AllowancesStorage, balances::BalancesStorage, meta::MetaStorage,
    total_supply::TotalSupplyStorage,
};
use core::{cmp::Ordering, fmt::Debug};
use gstd::{ext, format, msg, ActorId, Decode, Encode, String, TypeInfo, Vec};
use primitive_types::U256;
use sails_macros::gservice;
use sails_rtl::gstd::events::EventTrigger;

pub struct Service<X> {
    informer: X,
}

impl<X> Service<X> {
    pub fn seed(name: String, symbol: String, decimals: u8) {
        let _res = AllowancesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = BalancesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = MetaStorage::set(name, symbol, decimals);
        debug_assert!(_res.is_ok());
    }
}

impl<X: EventTrigger<Event>> Service<X> {
    pub fn deposit_event(&self, e: Event) {
        // TODO (sails): rename to `deposit_event`
        // TODO (sails): make infallible or something?
        if self.informer.trigger(e).is_err() {
            panic("Failed to deposit event");
        }
    }
}

// TODO (sails): consider renaming `EventTrigger` -> `Notifier`/`Informer`.
// TODO (sails): fix that requires `Encode`, `Decode`, `TypeInfo` and `Vec` in scope.
// TODO (sails): fix that requires explicit `-> ()`.
// TODO (sails): let me specify error as subset of strings (Display of my Error) -> thats common flow for us.
// TODO (sails): fix bug with unreachable names.
// TODO (sails): gstd::ActorId, primitive_types::H256/U256, [u8; 32], NonZeroStuff are primitives!.
#[gservice]
impl<X: EventTrigger<Event>> Service<X> {
    // TODO (sails): hide this into macro.
    pub fn new(informer: X) -> Self {
        Self { informer }
    }

    pub fn allowance(&self, owner: ActorId, spender: ActorId) -> U256 {
        allowance(AllowancesStorage::get(), owner, spender)
    }

    pub fn approve(&mut self, spender: ActorId, value: U256) -> bool {
        let owner = msg::source();

        let mutated = approve(AllowancesStorage::get_mut(), owner, spender, value);

        if mutated {
            self.deposit_event(Event::Approval {
                owner,
                spender,
                value,
            })
        }

        mutated
    }

    pub fn balance_of(&self, owner: ActorId) -> U256 {
        balance_of(BalancesStorage::get(), owner)
    }

    pub fn decimals(&self) -> u8 {
        MetaStorage::decimals()
    }

    // TODO (sails): allow using references.
    pub fn name(&self) -> String {
        MetaStorage::name()
    }

    pub fn symbol(&self) -> String {
        MetaStorage::symbol()
    }

    pub fn total_supply(&self) -> U256 {
        TotalSupplyStorage::get()
    }

    pub fn transfer(&mut self, to: ActorId, value: U256) -> bool {
        let from = msg::source();

        let mutated = panicking(move || transfer(BalancesStorage::get_mut(), from, to, value));

        if mutated {
            let value = value
                .try_into()
                .expect("Infallible since `transfer` executed successfully");

            self.deposit_event(Event::Transfer { from, to, value })
        }

        mutated
    }

    // TODO (breathx): rename me once bug in sails fixed.
    pub fn from_transfer(&mut self, from: ActorId, to: ActorId, value: U256) -> bool {
        let spender = msg::source();

        let mutated = panicking(move || {
            transfer_from(
                AllowancesStorage::get_mut(),
                BalancesStorage::get_mut(),
                spender,
                from,
                to,
                value,
            )
        });

        if mutated {
            let value = value
                .try_into()
                .expect("Infallible since `transfer_from` executed successfully");

            self.deposit_event(Event::Transfer { from, to, value })
        }

        mutated
    }

    // TODO (breathx): delete me once multi services are implemented.
    pub fn set_balance(&mut self, new_balance: U256) -> bool {
        let owner = msg::source();

        let balance = balance_of(BalancesStorage::get(), owner);

        let new_total_supply = match balance.cmp(&new_balance) {
            Ordering::Greater => TotalSupplyStorage::get().saturating_sub(balance - new_balance),
            Ordering::Less => TotalSupplyStorage::get().saturating_add(new_balance - balance),
            Ordering::Equal => return false,
        };

        let non_zero_new_balance = new_balance
            .try_into()
            .expect("Infallible since NonZero b/c new_balance != balance");

        BalancesStorage::get_mut().insert(owner, non_zero_new_balance);
        *TotalSupplyStorage::get_mut() = new_total_supply;

        true
    }
}

fn panicking<T, F: FnOnce() -> Result<T, Error>>(f: F) -> T {
    match f() {
        Ok(v) => v,
        Err(e) => panic(e),
    }
}

fn panic(err: impl Debug) -> ! {
    ext::panic(&format!("{err:?}"))
}
