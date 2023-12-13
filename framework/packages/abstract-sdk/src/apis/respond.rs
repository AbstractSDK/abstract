use crate::features::{CustomEvents, ModuleIdentification};
use cosmwasm_std::Attribute;

/// Wrap a [`Response`] with an abstract-specific event.
/// The event contains the contract name and the action, and any additional attributes.
pub trait AbstractResponse: ModuleIdentification + CustomEvents {
    /// Respond with an abstract-specific event that contains the contract name and the action.
    fn tag_response(&mut self, action: impl Into<String>) {
        self.custom_tag_response(action, Vec::<Attribute>::new())
    }
    /// `.tag_response()` with additional attributes.
    fn custom_tag_response(
        &mut self,
        action: impl Into<String>,
        attributes: impl IntoIterator<Item = impl Into<Attribute>>,
    ) {
        let module_id = self.module_id().to_string();

        self.add_event(
            "abstract",
            vec![("contract", module_id), ("action", action.into())],
        );
        self.add_attributes(attributes);
    }
}

impl<T> AbstractResponse for T where T: ModuleIdentification + CustomEvents {}
