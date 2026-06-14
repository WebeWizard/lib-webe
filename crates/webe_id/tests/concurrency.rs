mod common;

use std::collections::HashSet;
use std::mem;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use common::{StepClock, at_ms, epoch};
use webe_id::{Generator, NodeId, WebeId};

#[test]
fn documented_shared_generation_pattern_produces_unique_ids() {
    const WORKERS: usize = 8;
    const IDS_PER_WORKER: usize = 12_500;

    let clock = StepClock::new(at_ms(1), Duration::from_millis(1));
    let generator = Generator::builder(NodeId::from_u8(11))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();
    let shared = Arc::new(Mutex::new(generator));

    let handles = (0..WORKERS)
        .map(|_| {
            let shared = Arc::clone(&shared);
            thread::spawn(move || {
                (0..IDS_PER_WORKER)
                    .map(|_| shared.lock().unwrap().generate().unwrap())
                    .collect::<Vec<WebeId>>()
            })
        })
        .collect::<Vec<_>>();

    let mut ids = HashSet::with_capacity(WORKERS * IDS_PER_WORKER);
    for handle in handles {
        for id in handle.join().unwrap() {
            assert!(ids.insert(id));
        }
    }

    assert_eq!(ids.len(), 100_000);
}

#[test]
fn generator_owned_memory_is_fixed_size_during_volume_generation() {
    let clock = StepClock::new(at_ms(1), Duration::from_millis(1));
    let mut generator = Generator::builder(NodeId::from_u8(12))
        .with_epoch(epoch())
        .with_clock(Arc::new(clock))
        .build()
        .unwrap();
    let generator_size = mem::size_of_val(&generator);

    for _ in 0..1_000_000 {
        let _ = generator.generate().unwrap();
    }

    assert_eq!(mem::size_of_val(&generator), generator_size);
    assert!(generator_size <= 128);
}
