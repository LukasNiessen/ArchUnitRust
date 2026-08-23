pub mod model;

pub struct Handler;

#[std::prelude]
impl crate::shared::Port for Handler {
    fn handle(
        input: std::collections::HashMap<String, crate::shared::Marker>,
    ) -> core::option::Option<crate::api::model::Model> {
        let crate::shared::Marker::Active = crate::shared::marker();
        std::println!("{}", input.len());
        Some(crate::api::model::Model)
    }
}
