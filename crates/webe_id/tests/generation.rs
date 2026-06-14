mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{ManualClock, StepClock, at_ms, epoch};
use webe_id::{Generator, NodeId, WebeId};

#[test]
fn generated_ids_follow_layout_and_recompose() {
    let clock = ManualClock::new(at_ms(42));
    let mut generator = Generator::builder(NodeId::from_u8(7))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();

    let id = generator.generate().unwrap();
    let components = id.components();

    assert_eq!(components.time_millis(), 42);
    assert_eq!(components.node_id(), NodeId::from_u8(7));
    assert_eq!(components.sequence(), 0);
    assert_eq!(id.as_u64(), (42_u64 << 24) | (7_u64 << 16));
    assert_eq!(components.to_id(), id);
}

#[test]
fn generated_ids_sort_by_time_component() {
    let clock = StepClock::new(at_ms(1), Duration::from_millis(1));
    let mut generator = Generator::builder(NodeId::from_u8(9))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();

    let ids = (0..10_000)
        .map(|_| generator.generate().unwrap())
        .collect::<Vec<_>>();
    let mut sorted = ids.clone();
    sorted.sort();

    assert_eq!(sorted, ids);
}

#[test]
fn same_millisecond_sequence_advances() {
    let clock = ManualClock::new(at_ms(100));
    let mut generator = Generator::builder(NodeId::from_u8(1))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();

    let first = generator.generate().unwrap().components();
    let second = generator.generate().unwrap().components();
    let third = generator.generate().unwrap().components();

    assert_eq!(first.sequence(), 0);
    assert_eq!(second.sequence(), 1);
    assert_eq!(third.sequence(), 2);
    assert_eq!(first.time_millis(), second.time_millis());
    assert_eq!(second.time_millis(), third.time_millis());
}

#[test]
fn node_component_differentiates_generators() {
    let clock = ManualClock::new(at_ms(250));
    let mut generator_a = Generator::builder(NodeId::from_u8(2))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock.clone()))
        .build()
        .unwrap();
    let mut generator_b = Generator::builder(NodeId::from_u8(3))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();

    let id_a = generator_a.generate().unwrap();
    let id_b = generator_b.generate().unwrap();

    assert_ne!(id_a, id_b);
    assert_eq!(id_a.components().node_id(), NodeId::from_u8(2));
    assert_eq!(id_b.components().node_id(), NodeId::from_u8(3));
}

#[test]
fn sequence_resets_when_time_advances() {
    let clock = ManualClock::new(at_ms(500));
    let mut generator = Generator::builder(NodeId::from_u8(4))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock.clone()))
        .build()
        .unwrap();

    assert_eq!(generator.generate().unwrap().components().sequence(), 0);
    assert_eq!(generator.generate().unwrap().components().sequence(), 1);

    clock.advance(Duration::from_millis(1));

    let id = generator.generate().unwrap();
    assert_eq!(id.components().time_millis(), 501);
    assert_eq!(id.components().sequence(), 0);
}

#[test]
fn raw_values_can_be_treated_as_canonical_ids() {
    let id = WebeId::from_raw(123);

    assert_eq!(id.as_u64(), 123);
}
