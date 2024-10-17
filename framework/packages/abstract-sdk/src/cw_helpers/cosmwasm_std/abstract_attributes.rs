use abstract_std::ABSTRACT_EVENT_TYPE;
use cosmwasm_std::{Attribute, Event, Response};

/// Trait for adding `abstract` attributes to something
pub trait AbstractAttributes {
    /// Add the provided `attrs`
    fn add_abstract_attributes<A: Into<Attribute>>(
        self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self;
}

/// Trait for adding `abstract` events to a [`Response`]
impl AbstractAttributes for Response {
    fn add_abstract_attributes<A: Into<Attribute>>(
        mut self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self {
        // Find the index of the first abstract event in the events vector
        let index = self.events.iter().position(|e| e.ty == ABSTRACT_EVENT_TYPE);

        if let Some(index) = index {
            // If an abstract event exists, replace it with a new event eq has the additional attributes
            let event = self.events.remove(index);
            let new_event = event.add_attributes(attrs);
            self.events.insert(index, new_event);
        } else {
            // If an abstract event does not exist, create a new one with the additional attributes
            let new_event = Event::new(ABSTRACT_EVENT_TYPE).add_attributes(attrs);
            self.events.push(new_event);
        }

        self
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    #[coverage_helper::test]
    fn test_add_abstract_attributes_no_abstract_event() {
        let mut response = Response::default();

        assert!(response.events.is_empty());

        response = response.add_abstract_attributes(vec![("key1", "value1")]);

        assert_eq!(response.events.len(), 1);
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, ABSTRACT_EVENT_TYPE.to_string());
        assert_eq!(event.attributes.len(), 1);

        let attr = event.attributes.first().unwrap();
        assert_eq!(attr.key, "key1".to_string());
        assert_eq!(attr.value, "value1".to_string());
    }

    #[coverage_helper::test]
    fn test_add_abstract_attributes_with_abstract_event() {
        const KEY_1: &str = "abstract_key1";
        const VALUE_1: &str = "abstract_value1";
        let mut response = Response::default()
            .add_event(Event::new(ABSTRACT_EVENT_TYPE).add_attributes(vec![(KEY_1, VALUE_1)]));

        assert_eq!(response.events.len(), 1);
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, ABSTRACT_EVENT_TYPE.to_string());
        assert_eq!(event.attributes.len(), 1);

        // Add the attributes
        const KEY_2: &str = "abstract_key2";
        const VALUE_2: &str = "abstract_value2";
        response = response.add_abstract_attributes(vec![(KEY_2, VALUE_2)]);

        assert_eq!(response.events.len(), 1);
        let event = response.events.first().unwrap();
        assert_eq!(event.ty, ABSTRACT_EVENT_TYPE.to_string());
        // should now be 2
        assert_eq!(event.attributes.len(), 2);

        let attr1 = event.attributes.first().unwrap();
        assert_eq!(attr1.key, KEY_1.to_string());
        assert_eq!(attr1.value, VALUE_1.to_string());

        let attr2 = event.attributes.get(1).unwrap();
        assert_eq!(attr2.key, KEY_2.to_string());
        assert_eq!(attr2.value, VALUE_2.to_string());
    }
}
