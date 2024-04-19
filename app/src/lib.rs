#![no_std]
#![allow(clippy::new_without_default)]
#![allow(dead_code)]

// TODO (sails): rename here to use `notifier`::`Notifier`/`Informer`.
mod informer {
    pub use sails_rtl::gstd::events::GStdEventTrigger as Gstd;
}
use gstd::String;
use sails_macros::{gprogram, groute};
use services::erc20;

pub mod services;

pub struct BreathxProgram;

// TODO (sails): allow to import all necessary macros at once (gprogram, grout, etc).
// TODO (sails): stop forcing deriving default on `BreathxProgram`.
#[gprogram]
impl BreathxProgram {
    // TODO (sails): fix arguments are unused.
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        <erc20::Service<()>>::seed(name, symbol, decimals);
        Self
    }

    #[groute("")]
    pub fn erc20(&self) -> erc20::Service<informer::Gstd<erc20::Event>> {
        erc20::Service::new(informer::Gstd::new())
    }
}
