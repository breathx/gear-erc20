use gstd::{collections::HashMap, prelude::*, ActorId, Decode, Encode, TypeInfo};
use primitive_types::U256;

pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    NumericOverflow,
    MaxSupplyReached,
    InsufficientBalance,
    Underflow,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct Init {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub description: String,
    pub external_links: ExternalLinks,
    pub initial_supply: U256,
    pub max_supply: U256,
    pub admin_id: ActorId,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct ExternalLinks {
    pub image: String,
    pub website: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub discord: Option<String>,
    pub tokenomics: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Role {
    Admin,
    Burner,
    Minter,
}
