//! End-to-end tests for the garden. Each test creates a fresh temp garden,
//! sets HORTUS_ROOT, and exercises one of the high-level commands.
//!
//! Tests share the HORTUS_ROOT env var, so a global mutex serializes them.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static COUNTER: AtomicUsize = AtomicUsize::new(0);
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn unique_temp_garden() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("hortus-test-{}-{}-{}", pid, n, "garden"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp garden");
    dir
}

fn with_garden<F: FnOnce()>(f: F) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = unique_temp_garden();
    std::env::set_var("HORTUS_ROOT", &dir);
    f();
    std::env::remove_var("HORTUS_ROOT");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn discover_creates_layout() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = unique_temp_garden();
    std::env::set_var("HORTUS_ROOT", &dir);
    let g = hortus::model::Garden::discover(None).expect("discover");
    assert!(g.seeds_dir().is_dir());
    assert!(g.beds_dir().is_dir());
    assert!(g.compost_dir().is_dir());
    assert!(g.cache_dir().is_dir());
    std::env::remove_var("HORTUS_ROOT");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_and_load_seed_roundtrip() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        let s = hortus::model::Seed {
            id: "2026-06-29-test-seed".into(),
            planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
            last_tended: None,
            mood: Some("curious".into()),
            tags: vec!["a".into(), "b".into()],
            composted_at: None,
            epitaph: None,
            body: "the body of the seed".into(),
            is_composted: false,
        };
        s.save(&g).unwrap();
        let loaded = hortus::model::Seed::load(&s.file_path(&g)).unwrap();
        assert_eq!(loaded.id, s.id);
        assert_eq!(loaded.body.trim(), s.body);
        assert_eq!(loaded.mood.as_deref(), Some("curious"));
        assert_eq!(loaded.tags, vec!["a", "b"]);
    });
}

#[test]
fn compost_moves_file_and_restores() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        let s = hortus::model::Seed {
            id: "2026-06-29-foo".into(),
            planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
            last_tended: None,
            mood: None,
            tags: vec![],
            composted_at: None,
            epitaph: None,
            body: "thought".into(),
            is_composted: false,
        };
        s.save(&g).unwrap();
        assert!(g.seeds_dir().join("2026-06-29-foo.md").exists());

        // Compost it.
        let mut s2 = hortus::model::Seed::load(&g.seeds_dir().join("2026-06-29-foo.md")).unwrap();
        s2.is_composted = true;
        s2.composted_at = Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
        s2.epitaph = Some("released".into());
        s2.save(&g).unwrap();
        assert!(!g.seeds_dir().join("2026-06-29-foo.md").exists());
        assert!(g.compost_dir().join("2026-06-29-foo.md").exists());

        // Restore.
        let mut s3 = hortus::model::Seed::load(&g.compost_dir().join("2026-06-29-foo.md")).unwrap();
        s3.is_composted = false;
        s3.composted_at = None;
        s3.epitaph = None;
        s3.save(&g).unwrap();
        assert!(g.seeds_dir().join("2026-06-29-foo.md").exists());
        assert!(!g.compost_dir().join("2026-06-29-foo.md").exists());

        let reloaded = hortus::model::Seed::load(&g.seeds_dir().join("2026-06-29-foo.md")).unwrap();
        assert!(!reloaded.is_composted);
        assert!(reloaded.composted_at.is_none());
        assert!(reloaded.epitaph.is_none());
    });
}

#[test]
fn climate_save_and_load() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        let mut c = hortus::model::Climate::load_or_default(&g).unwrap();
        c.now.mood = Some("tender".into());
        c.now.reading = Some("Calvino".into());
        c.now.season = Some("summer".into());
        c.save(&g).unwrap();

        let loaded = hortus::model::Climate::load_or_default(&g).unwrap();
        assert_eq!(loaded.now.mood.as_deref(), Some("tender"));
        assert_eq!(loaded.now.reading.as_deref(), Some("Calvino"));
        assert_eq!(loaded.now.season.as_deref(), Some("summer"));
    });
}

#[test]
fn bed_save_and_load() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        let bed = hortus::model::Bed {
            name: "test bed".into(),
            seeds: vec!["a".into(), "b".into()],
            description: "for testing".into(),
        };
        bed.save(&g).unwrap();
        let loaded = hortus::model::Bed::load(&bed.file_path(&g)).unwrap();
        assert_eq!(loaded.name, "test bed");
        assert_eq!(loaded.seeds, vec!["a", "b"]);
        assert_eq!(loaded.description, "for testing");
        assert_eq!(hortus::model::Bed::slug(&loaded.name), "test-bed");
    });
}

#[test]
fn load_all_seeds_includes_composted() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        for (i, compost) in [false, true, false].iter().enumerate() {
            let mut s = hortus::model::Seed {
                id: format!("2026-06-29-s{}", i),
                planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
                last_tended: None,
                mood: None,
                tags: vec![],
                composted_at: None,
                epitaph: None,
                body: format!("body {}", i),
                is_composted: false,
            };
            if *compost {
                s.is_composted = true;
                s.composted_at = Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
            }
            s.save(&g).unwrap();
        }
        let all = hortus::model::load_all_seeds(&g).unwrap();
        assert_eq!(all.len(), 3);
        let live = hortus::model::load_live_seeds(&g).unwrap();
        assert_eq!(live.len(), 2);
        assert!(live.iter().all(|s| !s.is_composted));
    });
}

#[test]
fn unique_seed_id_disambiguates() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        let s = hortus::model::Seed {
            id: "2026-06-29-foo".into(),
            planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
            last_tended: None,
            mood: None,
            tags: vec![],
            composted_at: None,
            epitaph: None,
            body: "x".into(),
            is_composted: false,
        };
        s.save(&g).unwrap();
        let id1 = hortus::model::unique_seed_id(&g, "2026-06-29-foo");
        assert_eq!(id1, "2026-06-29-foo-2");

        // Add another with the -2 id
        let s2 = hortus::model::Seed {
            id: "2026-06-29-foo-2".into(),
            planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
            last_tended: None,
            mood: None,
            tags: vec![],
            composted_at: None,
            epitaph: None,
            body: "y".into(),
            is_composted: false,
        };
        s2.save(&g).unwrap();
        let id2 = hortus::model::unique_seed_id(&g, "2026-06-29-foo");
        assert_eq!(id2, "2026-06-29-foo-3");
    });
}

#[test]
fn cross_finds_shared_rare_words() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        for (id, body) in [
            (
                "2026-06-29-cipher",
                "the cipher sits in the garden at midnight",
            ),
            (
                "2026-06-29-palimpsest",
                "a palimpsest of small notes about midnight",
            ),
        ] {
            let s = hortus::model::Seed {
                id: id.into(),
                planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
                last_tended: None,
                mood: None,
                tags: vec![],
                composted_at: None,
                epitaph: None,
                body: body.into(),
                is_composted: false,
            };
            s.save(&g).unwrap();
        }
        let seeds = hortus::model::load_live_seeds(&g).unwrap();
        let tokens: Vec<Vec<String>> = seeds
            .iter()
            .map(|s| hortus::text::tokenize(&s.body))
            .collect();
        let df = hortus::text::document_frequencies(tokens.iter().map(|v| v.as_slice()));
        assert!(df.contains_key("cipher"));
        assert!(df.contains_key("palimpsest"));
        // "midnight" appears in both — its df is 2
        assert_eq!(df.get("midnight").copied(), Some(2));
    });
}

#[test]
fn merge_unmerge_roundtrip() {
    with_garden(|| {
        let g = hortus::model::Garden::discover(None).unwrap();
        for (id, body) in [
            ("2026-06-29-first", "The first thought, small and quiet."),
            (
                "2026-06-29-second",
                "The second thought, in a different season.",
            ),
        ] {
            let s = hortus::model::Seed {
                id: id.into(),
                planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
                last_tended: None,
                mood: Some("curious".into()),
                tags: vec!["a".into()],
                composted_at: None,
                epitaph: None,
                body: body.into(),
                is_composted: false,
            };
            s.save(&g).unwrap();
            // Simulate what `merge` does: compost the originals.
            let mut composted = s.clone();
            composted.is_composted = true;
            composted.composted_at = Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap());
            composted.epitaph = Some("merged into `2026-06-29-merged`".to_string());
            composted.save(&g).unwrap();
        }
        // Manually create a merged seed in the same format merge.rs uses.
        let body = "> merged from `2026-06-29-first` (2026-06-29) and `2026-06-29-second` (2026-06-29).\n\n\
             ## from 2026-06-29-first\n\n\
             The first thought, small and quiet.\n\n\
             ## from 2026-06-29-second\n\n\
             The second thought, in a different season.\n"
            .to_string();
        let merged = hortus::model::Seed {
            id: "2026-06-29-merged".into(),
            planted: chrono::NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
            last_tended: None,
            mood: None,
            tags: vec![],
            composted_at: None,
            epitaph: None,
            body,
            is_composted: false,
        };
        merged.save(&g).unwrap();

        // Now unmerge it.
        hortus::cmd::unmerge::run(hortus::cmd::unmerge::UnmergeArgs {
            seed: "2026-06-29-merged".into(),
        })
        .expect("unmerge should succeed");

        // Originals should be back in seeds/
        assert!(g.seeds_dir().join("2026-06-29-first.md").exists());
        assert!(g.seeds_dir().join("2026-06-29-second.md").exists());
        assert!(!g.seeds_dir().join("2026-06-29-merged.md").exists());

        // Compost should be empty (besides the layouts).
        let composted: Vec<_> = fs::read_dir(g.compost_dir())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .collect();
        assert_eq!(composted.len(), 0);
    });
}
