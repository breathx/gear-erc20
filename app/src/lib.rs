#![no_std]
#![allow(clippy::new_without_default)]
#![allow(dead_code)]
#![allow(unused)]

use crate::admin::utils::Init;
use gstd::{msg, String};
use sails_macros::{gprogram, groute};
use sails_rtl::gstd::services::Service;
use services::{admin, aggregated, erc20, pausable, roles};

pub mod services;

type ServiceOf<T> = <T as sails_rtl::gstd::services::Service>::Exposure;

pub struct Program(());

#[gprogram]
impl Program {
    pub fn new(init: Init) -> Self {
        let roles_service = roles::GstdDrivenService::seed();
        let erc20_service = <erc20::GstdDrivenService>::seed(init.name, init.symbol, init.decimals);
        let pausable_service =
            <pausable::GstdDrivenService>::seed(roles_service.clone(), init.admin_id);
        let aggregated_service =
            <aggregated::GstdDrivenService>::seed(erc20_service, pausable_service);
        <admin::GstdDrivenService>::seed(
            roles_service,
            aggregated_service,
            init.admin_id,
            init.description,
            init.external_links,
            init.initial_supply,
            init.max_supply,
        );
        Self(())
    }

    #[groute("erc20")]
    pub fn admin(&self) -> admin::GstdDrivenService {
        admin::GstdDrivenService::new(self.roles(), self.aggregated())
    }

    fn aggregated(&self) -> aggregated::GstdDrivenService {
        aggregated::GstdDrivenService::new(self.erc20(), self.pausable())
    }

    // pub fn pausable(&self) -> pausable::GstdDrivenService {
    //     pausable::GstdDrivenService::new(self.roles())
    // }

    // pub fn roles(&self) -> roles::GstdDrivenService {
    //     roles::GstdDrivenService::new()
    // }

    fn erc20(&self) -> erc20::GstdDrivenService {
        erc20::GstdDrivenService::new()
    }
    fn roles(&self) -> roles::GstdDrivenService {
        roles::GstdDrivenService::new()
    }
    fn pausable(&self) -> pausable::GstdDrivenService {
        pausable::GstdDrivenService::new(self.roles())
    }
}
