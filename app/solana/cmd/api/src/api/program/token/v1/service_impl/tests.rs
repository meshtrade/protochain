#![allow(clippy::unwrap_used)]

use super::super::helpers::parse_human_amount;

#[test]
fn parse_human_amount_whole_number() {
    assert_eq!(parse_human_amount("1", 6).unwrap(), 1_000_000);
}

#[test]
fn parse_human_amount_with_decimals() {
    assert_eq!(parse_human_amount("1.5", 6).unwrap(), 1_500_000);
}

#[test]
fn parse_human_amount_exact_precision() {
    assert_eq!(parse_human_amount("0.000001", 6).unwrap(), 1);
}

#[test]
fn parse_human_amount_trailing_zeros_ok() {
    // "1.20" is fine for 2 decimals — trailing zeros don't add precision.
    assert_eq!(parse_human_amount("1.20", 2).unwrap(), 120);
}

#[test]
fn parse_human_amount_zero() {
    assert_eq!(parse_human_amount("0", 6).unwrap(), 0);
    assert_eq!(parse_human_amount("0.0", 6).unwrap(), 0);
}

#[test]
fn parse_human_amount_large_value() {
    assert_eq!(parse_human_amount("1000", 6).unwrap(), 1_000_000_000);
}

#[test]
fn parse_human_amount_rejects_excess_decimals() {
    // 1.2345 with a 2-decimal mint must NOT silently truncate to 1.23.
    let err = parse_human_amount("1.2345", 2).unwrap_err();
    let msg = err.message();
    assert!(msg.contains("more fractional digits"), "Expected precision error, got: {msg}");
    assert!(msg.contains("1.2345"), "Error should echo the input");
    assert!(msg.contains("1.23"), "Error should show what it would truncate to");
}

#[test]
fn parse_human_amount_rejects_negative() {
    let err = parse_human_amount("-1.0", 6).unwrap_err();
    assert!(err.message().contains("negative"));
}

#[test]
fn parse_human_amount_rejects_garbage() {
    assert!(parse_human_amount("abc", 6).is_err());
    assert!(parse_human_amount("", 6).is_err());
}

#[test]
fn parse_human_amount_zero_decimals_mint() {
    // NFT-style mint with 0 decimals — only whole numbers allowed.
    assert_eq!(parse_human_amount("42", 0).unwrap(), 42);
    assert!(parse_human_amount("1.5", 0).is_err());
}
