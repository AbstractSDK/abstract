#[cfg(test)]
mod tests {
    use abstract_macros::abstract_response;
    use cosmwasm_std::Response;
    use speculoos::prelude::*;

    const TEST_CONTRACT: &str = "test:contract";

    #[abstract_response(TEST_CONTRACT)]
    struct TestResponse;

    #[test]
    fn test_action() {
        let actual: Response = TestResponse::action("instantiate");
        let expected = Response::new().add_event(
            cosmwasm_std::Event::new("abstract")
                .add_attributes(vec![("contract", TEST_CONTRACT), ("action", "instantiate")]),
        );

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn test_new_with_attrs() {
        let actual: Response = TestResponse::new("action", vec![("custom", "abstract")]);

        let expected =
            Response::new().add_event(cosmwasm_std::Event::new("abstract").add_attributes(vec![
                ("contract", TEST_CONTRACT),
                ("action", "action"),
                ("custom", "abstract"),
            ]));

        assert_that!(actual).is_equal_to(expected);
    }
}
