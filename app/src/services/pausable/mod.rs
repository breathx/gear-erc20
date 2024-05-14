use crate::services::{self, pausable::roles::PauseAdmin, roles::storage::RolesStorage};
use core::marker::PhantomData;
use gstd::{msg, ActorId, Decode, Encode, String, TypeInfo, Vec};
use sails_macros::gservice;
use sails_rtl::gstd::events::{EventTrigger, GStdEventTrigger};
use storage::StateStorage;

pub use utils::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    Paused,
    Unpaused,
}

pub type GstdDrivenService = Service<GStdEventTrigger<Event>>;

#[derive(Clone)]
pub struct Service<X> {
    roles_service: services::roles::GstdDrivenService,
    _phantom: PhantomData<X>,
}

impl<X: EventTrigger<Event>> Service<X> {
    pub fn seed(mut roles_service: services::roles::GstdDrivenService, admin: ActorId) -> Self {
        let _res = StateStorage::set(State::Active);
        debug_assert!(_res.is_ok());

        roles_service.register_role::<PauseAdmin>();

        let _res = roles_service.grant_role::<PauseAdmin>(admin);
        debug_assert!(_res);

        Self {
            roles_service,
            _phantom: PhantomData,
        }
    }
}

#[gservice]
impl<X> Service<X>
where
    X: EventTrigger<Event>,
{
    pub fn new(roles_service: services::roles::GstdDrivenService) -> Self {
        Self {
            roles_service,
            _phantom: PhantomData,
        }
    }

    pub fn is_paused(&self) -> bool {
        StateStorage::as_ref().paused()
    }

    pub fn ensure_unpaused(&self) -> Result<(), Error> {
        (!self.is_paused()).then_some(()).ok_or(Error::Paused)
    }

    pub fn pause(&mut self) -> bool {
        services::utils::panicking(move || -> services::roles::Result<bool> {
            self.roles_service
                .ensure_has_role::<PauseAdmin>(msg::source())?;

            let mutated = funcs::pause(StateStorage::as_mut());

            if mutated {
                services::utils::deposit_event(Event::Paused);
            }

            Ok(mutated)
        })
    }

    pub fn unpause(&mut self) -> bool {
        services::utils::panicking(move || -> services::roles::Result<bool> {
            self.roles_service
                .ensure_has_role::<PauseAdmin>(msg::source())?;

            let mutated = funcs::unpause(StateStorage::as_mut());

            if mutated {
                services::utils::deposit_event(Event::Unpaused)
            }

            Ok(mutated)
        })
    }

    // TODO (breathx): consider as atomic
    pub fn delegate_admin(&mut self, actor: sails_rtl::ActorId) -> bool {
        services::utils::panicking(move || -> services::roles::Result<bool> {
            let source = msg::source();

            self.roles_service.ensure_has_role::<PauseAdmin>(source)?;

            if ActorId::from(actor) == source {
                return Ok(false);
            }

            let _res = self.roles_service.grant_role::<PauseAdmin>(actor.into());
            debug_assert!(_res);

            let _res = self.roles_service.remove_role::<PauseAdmin>(source);
            debug_assert!(_res);

            Ok(true)
        })
    }
}

pub mod funcs {
    use super::State;

    pub fn pause(state: &mut State) -> bool {
        if state.paused() {
            return false;
        }

        state.switch();

        true
    }

    pub fn unpause(state: &mut State) -> bool {
        if !state.paused() {
            return false;
        }

        state.switch();

        true
    }
}

pub mod roles {
    crate::declare_role!(PauseAdmin);
}

pub mod storage {
    use super::State;

    crate::declare_storage!(name: StateStorage, ty: State);
}

mod utils {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
    pub enum Error {
        Paused,
    }

    pub enum State {
        Active,
        Paused,
    }

    impl State {
        pub fn paused(&self) -> bool {
            matches!(self, Self::Paused)
        }

        pub fn switch(&mut self) {
            if self.paused() {
                *self = Self::Active
            } else {
                *self = Self::Paused
            }
        }
    }
}
