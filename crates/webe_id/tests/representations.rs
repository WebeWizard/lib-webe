use webe_id::{NodeId, ParseWebeIdError, WebeId, WebeIdComponents};

fn sample_id() -> WebeId {
    WebeIdComponents::new(0x0001_0203_0405, NodeId::from_u8(0xab), 0xcdef)
        .unwrap()
        .to_id()
}

#[test]
fn numeric_round_trip_preserves_components() {
    let id = sample_id();
    let round_trip = WebeId::from_raw(id.as_u64());

    assert_eq!(round_trip, id);
    assert_eq!(round_trip.components(), id.components());
}

#[test]
fn big_endian_byte_round_trip_preserves_layout() {
    let id = sample_id();
    let bytes = id.to_be_bytes();

    assert_eq!(bytes, [0x01, 0x02, 0x03, 0x04, 0x05, 0xab, 0xcd, 0xef]);
    assert_eq!(WebeId::from_be_bytes(bytes), id);
    assert_eq!(WebeId::parse_be_bytes(&bytes).unwrap(), id);
}

#[test]
fn decimal_text_round_trip_preserves_value() {
    let id = sample_id();
    let decimal = id.to_decimal_string();

    assert_eq!(WebeId::parse_decimal(&decimal).unwrap(), id);
    assert_eq!(decimal.parse::<WebeId>().unwrap(), id);
    assert_eq!(id.to_string(), decimal);
}

#[test]
fn hexadecimal_text_round_trip_preserves_value() {
    let id = sample_id();

    assert_eq!(id.to_hex_string(), "0102030405abcdef");
    assert_eq!(WebeId::parse_hex("0102030405abcdef").unwrap(), id);
    assert_eq!(WebeId::parse_hex("0x0102030405ABCDEF").unwrap(), id);
}

#[test]
fn malformed_representations_return_typed_errors() {
    assert!(matches!(
        WebeId::parse_be_bytes(&[1, 2, 3]),
        Err(ParseWebeIdError::InvalidByteLength { actual: 3, .. })
    ));
    assert!(matches!(
        WebeId::parse_decimal("not-a-number"),
        Err(ParseWebeIdError::InvalidDecimal { .. })
    ));
    assert!(matches!(
        WebeId::parse_hex("not-hex"),
        Err(ParseWebeIdError::InvalidHex { .. })
    ));
}
