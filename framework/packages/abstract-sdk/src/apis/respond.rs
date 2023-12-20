use crate::features::ModuleIdentification;
use abstract_macros::with_abstract_event;
use cosmwasm_std::{Attribute, Response};

/// Construct a [`Response`] with an abstract-specific event.
/// The event contains the contract name and the action, and any additional attributes.
pub trait AbstractResponse: ModuleIdentification {
    /// Respond with an abstract-specific event that contains the contract name and the action.
    fn tag_response(&self, action: impl Into<String>) -> Response {
        self.custom_tag_response(action, Vec::<Attribute>::new())
    }
    /// `.tag_response()` with additional attributes.
    fn custom_tag_response(
        &self,
        action: impl Into<String>,
        attributes: impl IntoIterator<Item = impl Into<Attribute>>,
    ) -> Response {
        let module_id = self.module_id();
        let response = Response::new();
        with_abstract_event!(response, module_id, action, attributes)
    }
}

impl<T> AbstractResponse for T where T: ModuleIdentification {}
