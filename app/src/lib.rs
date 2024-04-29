#![no_std]
#![allow(clippy::new_without_default)]
#![allow(dead_code)]
#![allow(unused)]

// TODO (sails): consider sub-services inside (for observing state, for example)
// TODO (sails): rename gstd event depositor here to use `notifier`::`Notifier`/`Informer`.
use gstd::{msg, String};
use sails_macros::{gprogram, groute};
use services::{admin, aggregated, erc20, pausable, roles};

pub mod services;

pub struct BreathxProgram(());

// TODO (sails): allow to import all necessary macros at once (gprogram, grout, etc).
// TODO (sails): stop forcing deriving default on `BreathxProgram`.
#[gprogram]
impl BreathxProgram {
    // TODO (sails): fix arguments are unused.
    // TODO (sails): `#[gconstructor]`
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        let source = msg::source();

        let roles_service = roles::GstdDrivenService::seed();

        let erc20_service = <erc20::GstdDrivenService>::seed(name, symbol, decimals);

        let pausable_service = <pausable::GstdDrivenService>::seed(roles_service.clone(), source);

        let aggregated_service =
            <aggregated::GstdDrivenService>::seed(erc20_service, pausable_service);

        <admin::GstdDrivenService>::seed(roles_service, aggregated_service, source);

        Self(())
    }

    #[groute]
    pub fn admin(&self) -> admin::GstdDrivenService {
        admin::GstdDrivenService::new(self.roles(), self.aggregated())
    }

    // TODO (sails): service Erc20: Pausable [pipeline]
    // TODO (sails): Should reflect on multiple names as pipeline (aliasing)
    #[groute("erc20")]
    pub fn aggregated(&self) -> aggregated::GstdDrivenService {
        aggregated::GstdDrivenService::new(self.erc20(), self.pausable())
    }

    #[groute]
    pub fn pausable(&self) -> pausable::GstdDrivenService {
        pausable::GstdDrivenService::new(self.roles())
    }

    #[groute]
    pub fn roles(&self) -> roles::GstdDrivenService {
        roles::GstdDrivenService::new()
    }

    fn erc20(&self) -> erc20::GstdDrivenService {
        erc20::GstdDrivenService::new()
    }
}
