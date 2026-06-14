mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{ManualClock, at_ms, epoch};
use webe_id::{
    BuildGeneratorError, GenerateError, Generator, MAX_TIME_MILLISECONDS, NodeId, NodeIdError,
    WebeIdComponents,
};

#[test]
fn generator_rejects_epoch_after_observed_time() {
    let clock = ManualClock::new(at_ms(10));
    let result = Generator::builder(NodeId::from_u8(1))
        .with_epoch(at_ms(11))
        .with_clock(Arc::new(clock))
        .build();

    assert!(matches!(result, Err(BuildGeneratorError::EpochInFuture)));
}

#[test]
fn generator_rejects_exhausted_time_range() {
    let clock = ManualClock::new(at_ms(MAX_TIME_MILLISECONDS + 1));
    let result = Generator::builder(NodeId::from_u8(1))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build();

    assert!(matches!(
        result,
        Err(BuildGeneratorError::TimeRangeExceeded { .. })
    ));
}

#[test]
fn node_validation_rejects_out_of_range_values() {
    assert!(matches!(
        NodeId::new(256),
        Err(NodeIdError::OutOfRange { value: 256 })
    ));
    assert!(matches!(
        Generator::builder_from_node_value(300),
        Err(BuildGeneratorError::InvalidNode(_))
    ));
}

#[test]
fn restart_marker_must_match_node() {
    let marker = WebeIdComponents::new(5, NodeId::from_u8(1), 10)
        .unwrap()
        .to_id();
    let clock = ManualClock::new(at_ms(6));

    let result = Generator::builder(NodeId::from_u8(2))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .with_restart_marker(marker)
        .build();

    assert!(matches!(
        result,
        Err(BuildGeneratorError::RestartMarkerNodeMismatch { .. })
    ));
}

#[test]
fn restart_marker_must_be_behind_current_duration() {
    let marker = WebeIdComponents::new(5, NodeId::from_u8(1), 10)
        .unwrap()
        .to_id();
    let clock = ManualClock::new(at_ms(5));

    let result = Generator::builder(NodeId::from_u8(1))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .with_restart_marker(marker)
        .build();

    assert!(matches!(
        result,
        Err(BuildGeneratorError::RestartMarkerNotBehindCurrentTime { .. })
    ));
}

#[test]
fn restart_marker_allows_safe_current_time() {
    let marker = WebeIdComponents::new(5, NodeId::from_u8(1), 10)
        .unwrap()
        .to_id();
    let clock = ManualClock::new(at_ms(6));

    let result = Generator::builder(NodeId::from_u8(1))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .with_restart_marker(marker)
        .build();

    assert!(result.is_ok());
}

#[test]
fn temporary_clock_rewind_fails_then_recovers() {
    let clock = ManualClock::new(at_ms(10));
    let mut generator = Generator::builder(NodeId::from_u8(1))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock.clone()))
        .build()
        .unwrap();

    assert_eq!(generator.generate().unwrap().components().time_millis(), 10);

    clock.set(at_ms(9));
    assert!(matches!(
        generator.generate(),
        Err(GenerateError::ClockRewind {
            observed_millis: 9,
            last_millis: 10
        })
    ));

    clock.set(at_ms(10));
    let recovered = generator.generate().unwrap();
    assert_eq!(recovered.components().time_millis(), 10);
    assert_eq!(recovered.components().sequence(), 1);
}

#[test]
fn sequence_capacity_exhaustion_fails_fast_and_recovers_after_time_advances() {
    let clock = ManualClock::new(at_ms(20));
    let mut generator = Generator::builder(NodeId::from_u8(1))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock.clone()))
        .build()
        .unwrap();

    for expected_sequence in 0..=u16::MAX {
        let id = generator.generate().unwrap();
        assert_eq!(id.components().sequence(), expected_sequence);
    }

    assert!(matches!(
        generator.generate(),
        Err(GenerateError::SequenceCapacityExhausted { time_millis: 20 })
    ));

    clock.advance(Duration::from_millis(1));
    let recovered = generator.generate().unwrap();
    assert_eq!(recovered.components().time_millis(), 21);
    assert_eq!(recovered.components().sequence(), 0);
}
