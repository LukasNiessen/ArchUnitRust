mod shared;
pub mod api;

#[path = "../alternate/storage.rs"]
mod storage;

#[cfg(target_os = "definitely-not-a-real-target")]
mod platform;

mod broken;
mod missing;
mod ambiguous;
mod legacy;

mod inline {
    mod nested;
    #[path = "redirected.rs"]
    mod redirected;

    pub fn marker() -> crate::shared::Marker {
        crate::shared::Marker::Active
    }
}

#[path = concat!(env!("OUT_DIR"), "/invalid.rs")]
mod invalid_path;

#[path = "library.rs"]
mod cycle;

pub use crate::api::{self as public_api, Handler};
use crate::shared::{self, value as shared_value};

extern crate proc_macro as tokens;
use macro_tools as macros_alias;
use wire_format as serialization;

mod ignore_cases {
    use ignored_inline::Thing; // archunit: ignore
    use crate::api as ignored_alias; // archunit: ignore
    // archunit: ignore ignored_preceding
    pub use ignored_preceding::Thing;
    use grouped::{Ignored, Kept}; // archunit: ignore grouped::Ignored
    extern crate ignored_extern; // archunit: ignore
    // archunit: ignore
    mod ignored_mod {
        pub fn still_parsed() {}
    }
    use mismatch::Thing; // archunit: ignore something_else
    use lookalike::Thing; // archunit ignore

    pub fn binding_still_resolves() -> ignored_alias::Handler {
        ignored_alias::Handler
    }
}

pub fn library_value() -> usize {
    use crate::api::model::Model as BlockModel;

    let _block_model: BlockModel = BlockModel;
    let _model: public_api::model::Model = public_api::model::Model;
    let _absolute: ::std::vec::Vec<usize> = ::std::vec::Vec::new();
    let _allocated: alloc::vec::Vec<usize> = alloc::vec::Vec::new();
    let _wire: serialization::Value = wire_format::Value;
    let _unknown: ghost_dependency::Thing;
    let _storage = crate::storage::load();
    let _platform = crate::platform::value();
    macros_alias::fixture!();
    tokio::join!(crate::macro_tokens_are_not_expanded::work());
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
    shared_value() + shared::value()
}
