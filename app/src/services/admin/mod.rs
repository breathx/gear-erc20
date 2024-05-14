use self::utils::Role;
use crate::services;
use crate::services::admin::roles::{FungibleAdmin, FungibleBurner, FungibleMinter};
use core::marker::PhantomData;
use gstd::{exec, msg, String};
use gstd::{ActorId, Decode, Encode, ToString, TypeInfo, Vec};
use primitive_types::U256;
use sails_macros::gservice;
use sails_rtl::gstd::events::{EventTrigger, GStdEventTrigger};

use super::erc20::storage::{AllowancesStorage, BalancesStorage, TotalSupplyStorage};
use crate::admin::utils::ExternalLinks;
use storage::AdditionalMetaStorage;
pub mod funcs;
pub mod storage;
pub(crate) mod utils;

pub type GstdDrivenService = Service<GStdEventTrigger<Event>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    Minted {
        to: sails_rtl::ActorId,
        value: U256,
    },
    Burned {
        from: sails_rtl::ActorId,
        value: U256,
    },
    Killed {
        inheritor: sails_rtl::ActorId,
    },
    TransferredToUsers {
        from: sails_rtl::ActorId,
        to: Vec<sails_rtl::ActorId>,
        value: U256,
    },
}

pub struct Service<X> {
    roles_service: services::roles::GstdDrivenService,
    aggregated_service: services::aggregated::GstdDrivenService,
    _phantom: PhantomData<X>,
}

impl<X: EventTrigger<Event>> Service<X> {
    pub fn seed(
        mut roles_service: services::roles::GstdDrivenService,
        aggregated_service: services::aggregated::GstdDrivenService,
        admin: ActorId,
        description: String,
        external_links: ExternalLinks,
        initial_supply: U256,
        max_supply: U256,
    ) -> Self {
        roles_service.register_role::<FungibleAdmin>();
        roles_service.register_role::<FungibleBurner>();
        roles_service.register_role::<FungibleMinter>();

        let _res = roles_service.grant_role::<FungibleAdmin>(admin);
        debug_assert!(_res);
        let _res = roles_service.grant_role::<FungibleBurner>(admin);
        debug_assert!(_res);
        let _res = roles_service.grant_role::<FungibleMinter>(admin);
        debug_assert!(_res);
        if initial_supply > max_supply {
            panic!("SupplyError");
        }

        if description.chars().count() > 500 {
            panic!("DescriptionError");
        }

        let _res = AdditionalMetaStorage::with_data(description, external_links, max_supply);
        debug_assert!(_res.is_ok());

        if !initial_supply.is_zero() {
            let balances = BalancesStorage::as_mut();
            let Ok(non_zero_initial_supply) = initial_supply.try_into() else {
                unreachable!("Infallible since fn is noop on zero value; qed");
            };
            balances.insert(admin, non_zero_initial_supply);
        }

        Self {
            roles_service,
            aggregated_service,
            _phantom: PhantomData,
        }
    }
}

#[gservice]
impl<X> Service<X>
where
    X: EventTrigger<Event>,
{
    pub fn new(
        roles_service: services::roles::GstdDrivenService,
        aggregated_service: services::aggregated::GstdDrivenService,
    ) -> Self {
        Self {
            roles_service,
            aggregated_service,
            _phantom: PhantomData,
        }
    }

    pub fn mint(&mut self, to: sails_rtl::ActorId, value: U256) -> bool {
        services::utils::panicking(|| self.aggregated_service.pausable_service.ensure_unpaused());

        self.roles_service
            .ensure_has_role::<FungibleMinter>(msg::source());

        let mutated = services::utils::panicking(|| {
            funcs::mint(
                BalancesStorage::as_mut(),
                AdditionalMetaStorage::as_ref(),
                TotalSupplyStorage::as_mut(),
                to.into(),
                value,
            )
        });

        if mutated {
            services::utils::deposit_event(Event::Minted { to, value });
        }

        mutated
    }

    pub fn burn(&mut self, from: sails_rtl::ActorId, value: U256) -> bool {
        services::utils::panicking(|| self.aggregated_service.pausable_service.ensure_unpaused());

        self.roles_service
            .ensure_has_role::<FungibleBurner>(msg::source());

        let mutated = services::utils::panicking(|| {
            funcs::burn(BalancesStorage::as_mut(), TotalSupplyStorage::as_mut(), from.into(), value)
        });

        if mutated {
            services::utils::deposit_event(Event::Burned { from, value });
        }

        mutated
    }

    pub fn transfer_to_users(&mut self, to: Vec<sails_rtl::ActorId>, value: U256) -> bool {
        let from = msg::source();
        let to_gstd: Vec<gstd::ActorId> =
            to.iter().map(|actor_id| actor_id.clone().into()).collect();
        let mutated = services::utils::panicking(|| {
            funcs::transfer_to_users(BalancesStorage::as_mut(), from.into(), to_gstd, value)
        });

        if mutated {
            services::utils::deposit_event(Event::TransferredToUsers {
                from: from.into(),
                to,
                value,
            });
        }

        mutated
    }

    pub fn allowances_reserve(&mut self, additional: u32) -> () {
        services::utils::panicking(|| self.aggregated_service.pausable_service.ensure_unpaused());

        funcs::allowances_reserve(AllowancesStorage::as_mut(), additional as usize)
    }

    pub fn balances_reserve(&mut self, additional: u32) -> () {
        services::utils::panicking(|| self.aggregated_service.pausable_service.ensure_unpaused());

        funcs::balances_reserve(BalancesStorage::as_mut(), additional as usize)
    }

    pub fn maps_data(&self) -> ((u32, u32), (u32, u32)) {
        let ((a_len, a_cap), (b_len, b_cap)) =
            funcs::maps_data(AllowancesStorage::as_ref(), BalancesStorage::as_ref());

        ((a_len as u32, a_cap as u32), (b_len as u32, b_cap as u32))
    }

    pub fn allowances(
        &self,
        skip: u32,
        take: u32,
    ) -> Vec<((sails_rtl::ActorId, sails_rtl::ActorId), U256)> {
        funcs::allowances(AllowancesStorage::as_ref(), skip as usize, take as usize)
            .into_iter()
            .map(|((id1, id2), v)| ((id1.into(), id2.into()), v.into()))
            .collect()
    }

    pub fn balances(&self, skip: u32, take: u32) -> Vec<(sails_rtl::ActorId, U256)> {
        funcs::balances(BalancesStorage::as_ref(), skip as usize, take as usize)
            .into_iter()
            .map(|(id, v)| (id.into(), v.into()))
            .collect()
    }

    pub fn grant_role(&mut self, to: sails_rtl::ActorId, role: Role) -> bool {
        services::utils::panicking(|| self.aggregated_service.pausable_service.ensure_unpaused());

        services::utils::panicking(|| -> Result<bool, services::roles::Error> {
            self.roles_service
                .ensure_has_role::<FungibleAdmin>(msg::source())?;

            let res = match role {
                Role::Admin => self.roles_service.grant_role::<FungibleAdmin>(to.into()),
                Role::Minter => self.roles_service.grant_role::<FungibleMinter>(to.into()),
                Role::Burner => self.roles_service.grant_role::<FungibleBurner>(to.into()),
            };

            Ok(res)
        })
    }

    pub fn remove_role(&mut self, from: sails_rtl::ActorId, role: Role) -> bool {
        services::utils::panicking(|| self.aggregated_service.pausable_service.ensure_unpaused());

        services::utils::panicking(|| -> Result<bool, services::roles::Error> {
            self.roles_service
                .ensure_has_role::<FungibleAdmin>(msg::source())?;

            let res = match role {
                Role::Admin => self.roles_service.remove_role::<FungibleAdmin>(from.into()),
                Role::Minter => self
                    .roles_service
                    .remove_role::<FungibleMinter>(from.into()),
                Role::Burner => self
                    .roles_service
                    .remove_role::<FungibleBurner>(from.into()),
            };

            Ok(res)
        })
    }

    pub fn kill(&mut self, inheritor: sails_rtl::ActorId) -> () {
        services::utils::panicking(|| {
            self.roles_service
                .ensure_has_role::<FungibleAdmin>(msg::source())
        });

        services::utils::deposit_event(Event::Killed { inheritor });

        exec::exit(inheritor.into())
    }

    pub fn description(&self) -> String {
        AdditionalMetaStorage::description()
    }

    pub fn external_links(&self) -> ExternalLinks {
        AdditionalMetaStorage::external_links()
    }

    pub fn max_supply(&self) -> U256 {
        AdditionalMetaStorage::max_supply()
    }
}

pub mod roles {
    crate::declare_role!(FungibleAdmin);
    crate::declare_role!(FungibleBurner);
    crate::declare_role!(FungibleMinter);
}
