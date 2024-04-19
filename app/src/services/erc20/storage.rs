// TODO (sails): impl such macro

use super::{AllowancesMap, BalancesMap};

pub mod balances {
    use super::*;

    pub struct BalancesStorage(());

    static mut INSTANCE: Option<BalancesMap> = None;

    impl BalancesStorage {
        pub fn is_set() -> bool {
            unsafe { INSTANCE.is_some() }
        }

        pub fn set(value: BalancesMap) -> Result<(), BalancesMap> {
            if Self::is_set() {
                Err(value)
            } else {
                unsafe { INSTANCE = Some(value) }
                Ok(())
            }
        }

        pub fn with_capacity(capacity: usize) -> Result<(), BalancesMap> {
            Self::set(BalancesMap::with_capacity(capacity))
        }

        pub fn default() -> Result<(), BalancesMap> {
            Self::with_capacity(u16::MAX as usize)
        }

        pub fn get() -> &'static BalancesMap {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { INSTANCE.as_ref().expect("Infallible b/c set above") }
        }

        pub fn get_mut() -> &'static mut BalancesMap {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { INSTANCE.as_mut().expect("Infallible b/c set above") }
        }
    }
}

pub mod allowances {
    use super::*;

    pub struct AllowancesStorage(());

    static mut INSTANCE: Option<AllowancesMap> = None;

    impl AllowancesStorage {
        pub fn is_set() -> bool {
            unsafe { INSTANCE.is_some() }
        }

        pub fn set(value: AllowancesMap) -> Result<(), AllowancesMap> {
            if Self::is_set() {
                Err(value)
            } else {
                unsafe { INSTANCE = Some(value) }
                Ok(())
            }
        }

        pub fn with_capacity(capacity: usize) -> Result<(), AllowancesMap> {
            Self::set(AllowancesMap::with_capacity(capacity))
        }

        pub fn default() -> Result<(), AllowancesMap> {
            Self::with_capacity(u16::MAX as usize)
        }

        pub fn get() -> &'static AllowancesMap {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { INSTANCE.as_ref().expect("Infallible b/c set above") }
        }

        pub fn get_mut() -> &'static mut AllowancesMap {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { INSTANCE.as_mut().expect("Infallible b/c set above") }
        }
    }
}

pub mod meta {
    use gstd::String;

    pub struct MetaStorage(());

    pub struct Meta {
        pub name: String,
        pub symbol: String,
        pub decimals: u8,
    }

    static mut INSTANCE: Option<Meta> = None;

    impl MetaStorage {
        pub fn is_set() -> bool {
            unsafe { INSTANCE.is_some() }
        }

        pub fn set(name: String, symbol: String, decimals: u8) -> Result<(), Meta> {
            let meta = Meta {
                name,
                symbol,
                decimals,
            };

            if Self::is_set() {
                Err(meta)
            } else {
                unsafe { INSTANCE = Some(meta) }
                Ok(())
            }
        }

        pub fn default() -> Result<(), Meta> {
            Self::set(String::from("Vara Network"), String::from("VARA"), 12)
        }

        pub fn get() -> &'static Meta {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { INSTANCE.as_ref().expect("Infallible b/c set above") }
        }

        pub fn name() -> String {
            Self::get().name.clone()
        }

        pub fn symbol() -> String {
            Self::get().symbol.clone()
        }

        pub fn decimals() -> u8 {
            Self::get().decimals
        }
    }
}

pub mod total_supply {
    use primitive_types::U256;

    pub struct TotalSupplyStorage(());

    static mut INSTANCE: Option<U256> = None;

    impl TotalSupplyStorage {
        pub fn is_set() -> bool {
            unsafe { INSTANCE.is_some() }
        }

        pub fn set(value: U256) -> Result<(), U256> {
            if Self::is_set() {
                Err(value)
            } else {
                unsafe { INSTANCE = Some(value) }
                Ok(())
            }
        }

        pub fn default() -> Result<(), U256> {
            Self::set(U256::zero())
        }

        pub fn get() -> U256 {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { *INSTANCE.as_ref().expect("Infallible b/c set above") }
        }

        pub fn get_mut() -> &'static mut U256 {
            if !Self::is_set() {
                let _res = Self::default();
                debug_assert!(_res.is_ok());
            }

            unsafe { INSTANCE.as_mut().expect("Infallible b/c set above") }
        }
    }
}
