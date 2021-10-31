use crate::contract::{
    MAX_DESC_LENGTH, MAX_LINK_LENGTH, MAX_TITLE_LENGTH, MIN_DESC_LENGTH, MIN_LINK_LENGTH,
    MIN_TITLE_LENGTH,
};
use crate::validators::{
    validate_decimal_value, validate_poll_description, validate_poll_link, validate_poll_title,
    validate_quorum, validate_threshold,
};
use cosmwasm_std::Decimal;
use std::str::FromStr;

/**
 * Tests [validate_decimal_value]
 */

/**
 * Tests [validate_decimal_value] with an invalid value, i.e. value > max_value.
 */
#[test]
#[should_panic]
fn invalid_validate_interval() {
    validate_decimal_value(
        Decimal::one(),
        Decimal::one() - Decimal::from_str("0.000000000000000001").unwrap(),
    )
    .unwrap();
}

/**
 * Tests [validate_decimal_value] with valid values, i.e. 0 <= value <= max_value.
 */
#[test]
fn valid_validate_interval() {
    let mut valid = validate_decimal_value(Decimal::one(), Decimal::one()).unwrap();
    assert_eq!(valid, ());
    valid = validate_decimal_value(Decimal::zero(), Decimal::one()).unwrap();
    assert_eq!(valid, ());
}

/**
 * Tests [validate_quorum]
 */

/**
 * Tests [validate_quorum] with a value higher than 1.
 */
#[test]
#[should_panic]
fn invalid_quorum_1() {
    validate_quorum(Decimal::from_ratio(2u128, 1u128)).unwrap();
}

/**
 * Tests [validate_quorum] with an infinite value.
 */
#[test]
#[should_panic]
fn invalid_quorum_2() {
    validate_quorum(Decimal::from_ratio(1u128, 0u128)).unwrap();
}

/**
 * Tests [validate_quorum] with valid values, i.e. between [0,1].
 */
#[test]
fn valid_quorum() {
    let mut valid = validate_quorum(Decimal::zero()).unwrap();
    assert_eq!(valid, ());

    valid = validate_quorum(Decimal::one()).unwrap();
    assert_eq!(valid, ());

    valid = validate_quorum(Decimal::from_ratio(1u128, 2u128)).unwrap();
    assert_eq!(valid, ());
}

/**
 * Tests [validate_threshold]
 */

/**
 * Tests [validate_threshold] with a value higher than 1.
 */
#[test]
#[should_panic]
fn invalid_threshold_1() {
    validate_threshold(Decimal::from_ratio(2u128, 1u128)).unwrap();
}

/**
 * Tests [validate_threshold] with an infinite value.
 */
#[test]
#[should_panic]
fn invalid_threshold_2() {
    validate_threshold(Decimal::from_ratio(1u128, 0u128)).unwrap();
}

/**
 * Tests [validate_threshold] with valid values, i.e. between [0,1].
 */
#[test]
fn valid_threshold() {
    let mut valid = validate_threshold(Decimal::zero()).unwrap();
    assert_eq!(valid, ());

    valid = validate_threshold(Decimal::one()).unwrap();
    assert_eq!(valid, ());

    valid = validate_threshold(Decimal::from_ratio(1u128, 2u128)).unwrap();
    assert_eq!(valid, ());
}

/**
 * Tests [validate_poll_link]
 */

/**
 * Tests [validate_poll_link] with an invalid short value, i.e. len() < MIN_LINK_LENGTH.
 */
#[test]
#[should_panic]
fn invalid_short_link() {
    let link = String::from("invalidlink");
    assert_eq!(link.len(), MIN_LINK_LENGTH - 1);
    validate_poll_link(&Some(link)).unwrap();
}

/**
 * Tests [validate_poll_link] with an invalid long value, i.e. len() > MAX_LINK_LENGTH.
 */
#[test]
#[should_panic]
fn invalid_long_link() {
    let link = String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut en");
    assert_eq!(link.len(), MAX_LINK_LENGTH + 1);
    validate_poll_link(&Some(link)).unwrap();
}

/**
 * Tests [validate_poll_link] with a valid value, i.e. len() >= MIN_LINK_LENGTH && len() <= MAX_LINK_LENGTH.
 */
#[test]
fn valid_link() {
    let link = String::from("this is a valid link");
    assert!(link.len() >= MIN_LINK_LENGTH && link.len() <= MAX_LINK_LENGTH);
    let valid = validate_poll_link(&Some(link)).unwrap();
    assert_eq!(valid, ());
}

/**
 * Tests [validate_poll_title]
 */

/**
 * Tests [validate_poll_title] with an invalid short value, i.e. len() < MIN_TITLE_LENGTH.
 */
#[test]
#[should_panic]
fn invalid_short_poll_title() {
    let title = String::from("abc");
    assert_eq!(title.len(), MIN_TITLE_LENGTH - 1);
    validate_poll_title(&title).unwrap();
}

/**
 * Tests [validate_poll_title] with an invalid long value, i.e. len() > MAX_TITLE_LENGTH.
 */
#[test]
#[should_panic]
fn invalid_long_poll_title() {
    let title = String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do e");
    assert_eq!(title.len(), MAX_TITLE_LENGTH + 1);
    validate_poll_title(&title).unwrap();
}

/**
 * Tests [validate_poll_title] with a valid value, i.e. i.e. len() >= MIN_TITLE_LENGTH && len() <= MAX_TITLE_LENGTH.
 */
#[test]
fn valid_poll_title() {
    let title = String::from("this is a valid title");
    assert!(title.len() >= MIN_TITLE_LENGTH && title.len() <= MAX_TITLE_LENGTH);
    let valid = validate_poll_title(&title).unwrap();
    assert_eq!(valid, ());
}

/**
 * Tests [validate_poll_description]
 */

/**
 * Tests [validate_poll_description] with an invalid short value, i.e. len() < MIN_DESC_LENGTH.
 */
#[test]
#[should_panic]
fn invalid_short_poll_description() {
    let desc = String::from("ibc");
    assert_eq!(desc.len(), MIN_DESC_LENGTH - 1);
    validate_poll_description(&desc).unwrap();
}

/**
 * Tests [validate_poll_description] with an invalid long value, i.e. len() > MAX_DESC_LENGTH.
 */
#[test]
#[should_panic]
fn invalid_long_poll_description() {
    let s1 = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
    tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
    exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor \
    in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur \
    sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est \
    laborum.";
    let s2 = "Section 1.10.32 of de Finibus Bonorum et Malorum, written by Cicero in 45 BC";
    let s3 = "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium \
    doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi \
    architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit \
    aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem \
    sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, \
    adipisci velit, sed quia non numquam eius modi tempora ";
    let desc = s1.to_owned().clone() + s2 + s3;

    assert_eq!(desc.len(), MAX_DESC_LENGTH + 1);
    validate_poll_description(&desc).unwrap();
}

/**
 * Tests [validate_poll_description] with a valid value, i.e. i.e. len() >= MIN_DESC_LENGTH && len() <= MAX_DESC_LENGTH.
 */
#[test]
fn valid_poll_description() {
    let desc = String::from("this is a valid title");
    assert!(desc.len() >= MIN_DESC_LENGTH && desc.len() <= MAX_DESC_LENGTH);
    let valid = validate_poll_description(&desc).unwrap();
    assert_eq!(valid, ());
}
