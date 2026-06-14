#![cfg(feature = "id")]

#[test]
fn id_facade_exposes_core_generation_api() {
    let mut generator = webe::id::Generator::builder(webe::id::NodeId::from_u8(1))
        .build()
        .unwrap();

    let id = generator.generate().unwrap();

    assert_eq!(id.components().node_id(), webe::id::NodeId::from_u8(1));
}

#[cfg(feature = "id-tokio")]
#[test]
fn id_tokio_facade_exposes_backpressure_api() {
    let _options = webe::id::async_backpressure::BackpressureOptions::new(
        std::time::Duration::from_millis(10),
        std::time::Duration::from_millis(1),
    );
}
