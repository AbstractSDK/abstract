use abstract_core::ABSTRACT_EVENT_TYPE;
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
            // If an abstract event exists, replace it with a new event that has the additional attributes
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
    use cosmwasm_std::{Event, Response};
    use speculoos::prelude::*;

    use super::*;

    #[test]
    fn test_add_abstract_attributes_no_abstract_event() {
        let mut response = Response::default();

        assert_that!(response.events).has_length(0);

        response = response.add_abstract_attributes(vec![("key1", "value1")]);

        assert_that!(response.events).has_length(1);
        let event = response.events.get(0).unwrap();
        assert_that!(event.ty).is_equal_to(ABSTRACT_EVENT_TYPE.to_string());
        assert_that!(event.attributes).has_length(1);

        let attr = event.attributes.get(0).unwrap();
        assert_that!(attr.key).is_equal_to("key1".to_string());
        assert_that!(attr.value).is_equal_to("value1".to_string());
    }

    #[test]
    fn test_add_abstract_attributes_with_abstract_event() {
        const KEY_1: &str = "abstract_key1";
        const VALUE_1: &str = "abstract_value1";
        let mut response = Response::default()
            .add_event(Event::new(ABSTRACT_EVENT_TYPE).add_attributes(vec![(KEY_1, VALUE_1)]));

        assert_that!(response.events).has_length(1);
        let event = response.events.get(0).unwrap();
        assert_that!(event.ty).is_equal_to(ABSTRACT_EVENT_TYPE.to_string());
        assert_that!(event.attributes).has_length(1);

        // Add the attributes
        const KEY_2: &str = "abstract_key2";
        const VALUE_2: &str = "abstract_value2";
        response = response.add_abstract_attributes(vec![(KEY_2, VALUE_2)]);

        assert_that!(response.events).has_length(1);
        let event = response.events.get(0).unwrap();
        assert_that!(event.ty).is_equal_to(ABSTRACT_EVENT_TYPE.to_string());
        // should now be 2
        assert_that!(event.attributes).has_length(2);

        let attr1 = event.attributes.get(0).unwrap();
        assert_that!(attr1.key).is_equal_to(KEY_1.to_string());
        assert_that!(attr1.value).is_equal_to(VALUE_1.to_string());

        let attr2 = event.attributes.get(1).unwrap();
        assert_that!(attr2.key).is_equal_to(KEY_2.to_string());
        assert_that!(attr2.value).is_equal_to(VALUE_2.to_string());
    }
}
