//! Stage 5 golden tests: damaged segment → examination stream → SDA filter.
//!
//! Exit criteria (DELIVERY_PLAN Stage 5):
//! - If Residiuum can recover it, SDA can examine it (Stage 2–3 salvage outputs).
//! - Profile field set matches SDA_PROFILE.
//! - Damaged segment → stream → SDA finds only verified islands / reports holes.
//!
//! Note: [`Store::open`] rewrites the active segment to drop incomplete tails.
//! Hole examination of active-file garbage therefore uses [`examine_bytes`] on
//! the raw file (or sealed-segment corruption, which open does not rewrite).

use residiuum_examine::{
    examine_bytes, examine_store, filter_holes, filter_units, filter_verified_complete, map_units,
    ExamineLimits,
};
use residiuum_format::SafetyLimits;
use residiuum_store::{DurabilityMode, Store};
use std::fs::{self, OpenOptions};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn clean_store_verified_islands_via_sda() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    store.put("early", b"one", DurabilityMode::Durable).unwrap();
    store.put("late", b"two", DurabilityMode::Durable).unwrap();

    let page = examine_store(&store, ExamineLimits::default()).unwrap();
    assert!(page.complete, "unbounded exam should be complete");
    assert!(page.uncertainty.is_empty());

    let events: Vec<_> = page
        .units
        .iter()
        .filter(|u| u.unit_kind == "event" && u.status == "verified-complete")
        .collect();
    assert!(
        events.len() >= 2,
        "expected at least two put events, got {}",
        events.len()
    );

    // Profile fixed field set on every unit.
    for u in &page.units {
        let json = u.to_json();
        let fields = json.get("$fields").expect("ExaminationUnit JSON is a prod");
        for key in [
            "unit_kind",
            "status",
            "store_id",
            "segment_id",
            "item_id",
            "event_id",
            "event_kind",
            "physical",
            "integrity",
            "envelope",
            "payload",
            "holes",
            "provenance",
            "uncertainty",
        ] {
            assert!(
                fields.get(key).is_some(),
                "missing profile field {key} on {:?}",
                u.unit_kind
            );
        }
    }

    let islands = filter_verified_complete(&page.units).unwrap();
    assert!(islands.iter().all(|u| u.status == "verified-complete"));
    assert!(islands.len() >= 2);

    let statuses = map_units(&page.units, "input<status>").unwrap();
    assert_eq!(statuses.len(), page.units.len());
    assert!(statuses.iter().any(|s| s == "verified-complete"));
}

#[test]
fn damaged_segment_sda_finds_islands_and_holes() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store.put("early", b"1", DurabilityMode::Durable).unwrap();
        store.seal_active().unwrap();
        store.put("late", b"2", DurabilityMode::Durable).unwrap();
    }

    // Corrupt middle of the sealed segment (OVERVIEW §16 / Stage 3 salvage style).
    let segments = path.join("segments");
    let seg_file = fs::read_dir(&segments)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|x| x.to_str()) == Some("dingo"))
        .expect("sealed segment");
    let mut bytes = fs::read(&seg_file).unwrap();
    if bytes.len() > 80 {
        let i = bytes.len() / 2;
        bytes[i] ^= 0xff;
        fs::write(&seg_file, &bytes).unwrap();
    }

    let store = Store::open(&path).unwrap();
    // Late item on active segment survives ordinary open.
    assert_eq!(store.get("late").unwrap().as_deref(), Some(b"2".as_slice()));

    let page = examine_store(&store, ExamineLimits::default()).unwrap();
    assert!(page.complete);

    let holes = filter_holes(&page.units).unwrap();
    let islands = filter_verified_complete(&page.units).unwrap();

    // SDA filter: verified islands only.
    let sda_islands = filter_units(&page.units, r#"input<status> = "verified-complete""#).unwrap();
    assert_eq!(sda_islands.len(), islands.len());

    // SDA filter: holes only.
    let sda_holes = filter_units(&page.units, r#"input<unit_kind> = "hole""#).unwrap();
    assert_eq!(sda_holes.len(), holes.len());

    // Must report holes when middle corruption produces non-verified regions,
    // and still surface verified islands from undamaged frames.
    assert!(
        !islands.is_empty(),
        "if Residiuum recovers frames, SDA must examine them as verified units"
    );
    assert!(
        !holes.is_empty(),
        "middle-byte corruption of a sealed segment must project hole units"
    );

    // Hole envelope.reason is examinable via SDA Map required projection (→ Ok).
    let reasons = map_units(&holes, r#"input<envelope><"reason">!"#).unwrap();
    assert!(
        reasons.iter().any(|r| {
            r.get("$type").and_then(|t| t.as_str()) == Some("ok")
                && r.get("$value").and_then(|v| v.as_str()).is_some()
        }),
        "hole envelope.reason must be examinable via SDA: {reasons:?}"
    );

    // Verified events remain distinct from holes (status tags not collapsed).
    assert!(islands.iter().all(|u| u.unit_kind != "hole"));
    assert!(holes.iter().all(|u| u.unit_kind == "hole"));
}

#[test]
fn incomplete_tail_still_examines_earlier_frames() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store
            .put("keep", b"alive", DurabilityMode::Durable)
            .unwrap();
    }
    // Append incomplete magic to the raw active file *without* reopening the
    // store (open would truncate the tail). Examine offline bytes.
    let active = path.join("active").join("active.dingo");
    let mut f = OpenOptions::new().append(true).open(&active).unwrap();
    f.write_all(b"DINGOFRM").unwrap();
    f.write_all(&[0u8; 40]).unwrap();
    f.sync_all().unwrap();
    drop(f);

    let raw = fs::read(&active).unwrap();
    let page = examine_bytes(
        "active/active.dingo",
        &raw,
        SafetyLimits::default(),
        ExamineLimits::default(),
    )
    .unwrap();

    let islands = filter_verified_complete(&page.units).unwrap();
    assert!(
        islands.iter().any(|u| u.unit_kind == "event"),
        "earlier complete frames must remain examination units"
    );
    let holes = filter_holes(&page.units).unwrap();
    assert!(!holes.is_empty(), "incomplete tail must surface as holes");

    // SDA selects verified put events when present.
    let with_payload = filter_units(
        &page.units,
        r#"input<status> = "verified-complete" and input<unit_kind> = "event""#,
    )
    .unwrap();
    assert!(!with_payload.is_empty());
}

#[test]
fn resource_limit_never_fake_empty_complete() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    for i in 0..8 {
        store
            .put(&format!("k{i}"), &[i as u8], DurabilityMode::Durable)
            .unwrap();
    }

    let page = examine_store(&store, ExamineLimits::default().max_units(2)).unwrap();
    assert!(!page.complete);
    assert_eq!(page.units.len(), 2);
    assert!(page.uncertainty.iter().any(|t| t == "resource-limited"));
    assert!(page.continuation.is_some());

    // SDA over the incomplete page still sees units and complete=false.
    let complete_flag = residiuum_examine::eval_page(&page, "input<complete>").unwrap();
    assert_eq!(complete_flag, serde_json::json!(false));
}

#[test]
fn ordering_is_deterministic_across_calls() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    store.put("a", b"1", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    store.put("b", b"2", DurabilityMode::Durable).unwrap();

    let p1 = examine_store(&store, ExamineLimits::default()).unwrap();
    let p2 = examine_store(&store, ExamineLimits::default()).unwrap();
    let keys = |p: &residiuum_examine::ExaminePage| {
        p.units
            .iter()
            .map(|u| {
                (
                    u.unit_kind.clone(),
                    u.status.clone(),
                    u.physical.source.clone(),
                    u.physical.offset,
                    u.event_id.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&p1), keys(&p2));
}

#[test]
fn null_vs_absence_on_opt_identities() {
    // Hole units have None identities → SDA Opt none, not Null.
    // Pure garbage source (no open/truncate path).
    let garbage = b"not-a-frame-at-all!!!!";
    let page = examine_bytes(
        "garbage.dingo",
        garbage,
        SafetyLimits::default(),
        ExamineLimits::default(),
    )
    .unwrap();
    let holes = filter_holes(&page.units).unwrap();
    assert!(!holes.is_empty());
    let item_id = map_units(&holes, "input<item_id>").unwrap();
    for v in item_id {
        assert_eq!(
            v.get("$type").and_then(|t| t.as_str()),
            Some("none"),
            "absent identity must be None, not Null: {v}"
        );
    }
}
