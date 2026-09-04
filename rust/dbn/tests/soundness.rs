//! Aliasing and provenance coverage for the raw-pointer record views.
//!
//! These assertions only prove anything under
//! `cargo +nightly miri test --test soundness`.

use dbn::{MboMsg, Record, RecordBuf, RecordMut, RecordRef, RecordRefMut};

#[repr(align(8))]
struct AlignedBuf([u8; 128]);

fn mbo_buf() -> AlignedBuf {
    let mut mbo = MboMsg::default();
    mbo.hd.instrument_id = 1;
    mbo.order_id = 2;
    let src = mbo.as_ref();
    let mut buf = AlignedBuf([0; 128]);
    buf.0[..src.len()].copy_from_slice(src);
    buf
}

#[test]
fn get_mut_writes_through_pointer_from_new() {
    let mut buf = mbo_buf();
    // Safety: 8-aligned and holds a valid `MboMsg`
    let mut rec = unsafe { RecordRefMut::new(&mut buf.0) };
    rec.get_mut::<MboMsg>().unwrap().order_id = 7;
    assert_eq!(rec.get::<MboMsg>().unwrap().order_id, 7);
}

#[test]
fn try_get_mut_writes_through_pointer_from_new() {
    let mut buf = mbo_buf();
    // Safety: 8-aligned and holds a valid `MboMsg`
    let mut rec = unsafe { RecordRefMut::new(&mut buf.0) };
    rec.try_get_mut::<MboMsg>().unwrap().order_id = 8;
    assert_eq!(rec.try_get::<MboMsg>().unwrap().order_id, 8);
}

#[test]
fn get_mut_unchecked_writes_through_pointer_from_new() {
    let mut buf = mbo_buf();
    // Safety: 8-aligned and holds a valid `MboMsg`
    let mut rec = unsafe { RecordRefMut::new(&mut buf.0) };
    // Safety: `has::<MboMsg>` holds for this buffer
    unsafe { rec.get_mut_unchecked::<MboMsg>() }.order_id = 9;
    assert_eq!(rec.get::<MboMsg>().unwrap().order_id, 9);
}

#[test]
fn header_mut_writes_through_pointer_from_new() {
    let mut buf = mbo_buf();
    // Safety: 8-aligned and holds a valid `MboMsg`.
    let mut rec = unsafe { RecordRefMut::new(&mut buf.0) };
    rec.header_mut().instrument_id = 11;
    assert_eq!(rec.instrument_id(), 11);
}

#[test]
fn get_mut_reaches_past_the_header_from_a_record_borrow() {
    let mut mbo = MboMsg::default();
    let mut rec = RecordRefMut::from(&mut mbo);
    let inner = rec.get_mut::<MboMsg>().unwrap();
    inner.hd.instrument_id = 21;
    inner.order_id = 22;
    inner.price = 23;
    assert_eq!(mbo.order_id, 22);
    assert_eq!(mbo.price, 23);
    assert_eq!(mbo.hd.instrument_id, 21);
}

#[test]
fn header_read_does_not_outlive_a_later_mutation() {
    let mut mbo = MboMsg::default();
    let mut rec = RecordRefMut::from(&mut mbo);
    let instrument_id = rec.header().instrument_id;
    rec.header_mut().instrument_id = instrument_id + 1;
    assert_eq!(rec.instrument_id(), u64::from(instrument_id) + 1);
}

#[test]
fn shared_view_reads_after_mutation_through_the_owner() {
    let mut mbo = MboMsg::default();
    let mut rec = RecordRefMut::from(&mut mbo);
    rec.get_mut::<MboMsg>().unwrap().order_id = 12;
    let shared: RecordRef = rec.as_rec_ref();
    assert_eq!(shared.get::<MboMsg>().unwrap().order_id, 12);
}

#[test]
fn record_buf_round_trips_through_a_mutable_view() {
    let mbo = MboMsg {
        order_id: 3,
        ..Default::default()
    };
    let mut buf: RecordBuf = RecordBuf::from(mbo);
    let mut rec = buf.as_rec_ref_mut();
    rec.get_mut::<MboMsg>().unwrap().order_id = 13;
    assert_eq!(buf.get::<MboMsg>().unwrap().order_id, 13);
}
