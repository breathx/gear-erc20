use super::utils::ExternalLinks;
use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub struct AdditionalMeta {
    pub description: String,
    pub external_links: ExternalLinks,
    pub max_supply: U256,
}

crate::declare_storage!(module: meta, name: AdditionalMetaStorage, ty: AdditionalMeta);

impl AdditionalMetaStorage {
    pub fn with_data(
        description: String,
        external_links: ExternalLinks,
        max_supply: U256,
    ) -> Result<(), AdditionalMeta> {
        Self::set(AdditionalMeta {
            description,
            external_links,
            max_supply,
        })
    }

    pub fn default() -> Result<(), AdditionalMeta> {
        Self::with_data(
            String::from("Vara Network"),
            ExternalLinks {
                image: String::from("VARA"),
                website: None,
                telegram: None,
                twitter: None,
                discord: None,
                tokenomics: None,
            },
            U256::max_value(),
        )
    }

    pub fn description() -> String {
        Self::as_ref().description.clone()
    }

    pub fn external_links() -> ExternalLinks {
        Self::as_ref().external_links.clone()
    }

    pub fn max_supply() -> U256 {
        Self::as_ref().max_supply
    }
}
