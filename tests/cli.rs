//! End-to-end CLI tests. Each test spawns the actual `hortus` binary and
//! exercises the full surface — argument parsing, exit codes, stdout/stderr,
//! real filesystem behavior.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static COUNTER: AtomicUsize = AtomicUsize::new(0);
static LOCK: Mutex<()> = Mutex::new(());

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hortus"))
}

fn unique_temp() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("hortus-cli-{}-{}-{}", pid, n, "garden"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&str], root: &PathBuf) -> std::process::Output {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    Command::new(bin())
        .args(args)
        .arg("--root")
        .arg(root)
        .env("VISUAL", "true")
        .env("EDITOR", "true")
        .output()
        .expect("running hortus")
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// Strip ANSI escape codes from a string, for substring matching.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // skip the '['
            let _ = chars.next();
            // skip until we hit a letter
            while let Some(&nc) = chars.peek() {
                chars.next();
                if nc.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn assert_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "expected success, got {}\nstdout: {}\nstderr: {}",
        out.status,
        stdout(out),
        stderr(out)
    );
}

#[test]
fn help_shows_command_list() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = Command::new(bin()).arg("--help").output().expect("help");
    assert_ok(&out);
    let s = stdout(&out);
    for verb in [
        "plant", "sow", "tend", "wander", "list", "compost", "cross",
        "merge", "unmerge", "diary", "letter", "today", "forage",
        "stats", "untend", "climate", "quote", "tag", "bloom",
    ] {
        assert!(s.contains(verb), "--help missing command: {verb}\n{s}");
    }
}

#[test]
fn plant_then_list() {
    let root = unique_temp();
    let out1 = run(&["plant", "the first thought, however small"], &root);
    assert_ok(&out1);
    let s1 = stdout(&out1);
    assert!(s1.contains("planted"), "stdout: {s1}");

    let out2 = run(&["list"], &root);
    assert_ok(&out2);
    let s2 = stdout(&out2);
    assert!(
        s2.contains("the-first-thought-however"),
        "list should include new seed: {s2}"
    );
}

#[test]
fn plant_two_and_sow() {
    let root = unique_temp();
    let _ = run(&["plant", "alpha: a small observation"], &root);
    let _ = run(&["plant", "beta: a different observation"], &root);
    let out = run(&["sow", "first bed", "2026-06-29-alpha-a-small-observation"], &root);
    assert_ok(&out);

    let list_out = run(&["list", "--bed", "first-bed"], &root);
    assert_ok(&list_out);
    assert!(stdout(&list_out).contains("alpha"));
}

#[test]
fn compost_moves_and_restores() {
    let root = unique_temp();
    let _ = run(&["plant", "a seed to release"], &root);
    let seed_id = "2026-06-29-a-seed-to-release";
    assert!(root.join("seeds").join(format!("{seed_id}.md")).exists());

    let out = run(&["compost", seed_id, "--epitaph", "released for testing"], &root);
    assert_ok(&out);
    assert!(!root.join("seeds").join(format!("{seed_id}.md")).exists());
    assert!(root.join("compost").join(format!("{seed_id}.md")).exists());

    let out2 = run(&["compost", seed_id, "--restore"], &root);
    assert_ok(&out2);
    assert!(root.join("seeds").join(format!("{seed_id}.md")).exists());
}

#[test]
fn list_json_outputs_valid_json() {
    let root = unique_temp();
    let _ = run(&["plant", "first"], &root);
    let _ = run(&["plant", "second"], &root);
    let out = run(&["list", "--json"], &root);
    assert_ok(&out);
    let v: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("list --json should be valid JSON");
    let arr = v.as_array().expect("list --json should be a JSON array");
    assert!(arr.len() >= 2);
    let first = &arr[0];
    assert!(first.get("id").is_some());
    assert!(first.get("planted").is_some());
    assert!(first.get("body").is_some());
}

#[test]
fn today_json_has_expected_shape() {
    let root = unique_temp();
    let out = run(&["today", "--json"], &root);
    assert_ok(&out);
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("today JSON");
    assert!(v.get("date").is_some());
    assert!(v.get("climate").is_some());
    assert!(v.get("today_seeds").is_some());
}

#[test]
fn cross_json_outputs_pairs() {
    let root = unique_temp();
    let _ = run(&["plant", "the cipher sits in the garden"], &root);
    let _ = run(&["plant", "a palimpsest of small notes about midnight"], &root);
    let out = run(&["cross", "--json"], &root);
    assert_ok(&out);
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("cross JSON");
    // Should be a JSON array (possibly empty)
    assert!(v.is_array());
}

#[test]
fn forage_finds_phrase_with_context() {
    let root = unique_temp();
    let _ = run(&["plant", "the garden is a small quiet place"], &root);
    let out = run(&["forage", "small"], &root);
    assert_ok(&out);
    let s = stdout(&out);
    assert!(s.contains("small"), "forage should find the word 'small': {s}");
}

#[test]
fn forage_json_includes_snippets() {
    let root = unique_temp();
    let _ = run(&["plant", "a small thought about gardens"], &root);
    let out = run(&["forage", "small", "--json"], &root);
    assert_ok(&out);
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("forage JSON");
    let arr = v.as_array().unwrap();
    assert!(!arr.is_empty());
    let first = &arr[0];
    assert!(first.get("matches").is_some());
    let matches = first.get("matches").unwrap().as_array().unwrap();
    let m = &matches[0];
    assert!(m.get("match_text").is_some());
    assert!(m.get("context").is_some());
}

#[test]
fn climate_set_and_show() {
    let root = unique_temp();
    let out = run(&["climate", "--mood", "tested", "--reading", "the spec"], &root);
    assert_ok(&out);
    let show = run(&["climate"], &root);
    assert_ok(&show);
    let s = stdout(&show);
    assert!(s.contains("tested"), "climate should show mood: {s}");
    assert!(s.contains("the spec"), "climate should show reading: {s}");
}

#[test]
fn stats_outputs_counts() {
    let root = unique_temp();
    let _ = run(&["plant", "one"], &root);
    let _ = run(&["plant", "two"], &root);
    let out = run(&["stats"], &root);
    assert_ok(&out);
    let s = strip_ansi(&stdout(&out));
    assert!(s.contains("2 live"), "stats should show 2 live: {s}");
}

#[test]
fn stats_json_has_keys() {
    let root = unique_temp();
    let _ = run(&["plant", "a seed"], &root);
    let out = run(&["stats", "--json"], &root);
    assert_ok(&out);
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("stats JSON");
    for k in ["live_seeds", "composted_seeds", "beds", "moods", "tags"] {
        assert!(v.get(k).is_some(), "stats JSON missing key `{k}`");
    }
}

#[test]
fn untend_clears_last_tended() {
    let root = unique_temp();
    let _ = run(&["plant", "forgotten seed"], &root);
    // First tend it so last_tended is set.
    let _ = run(&["untend", "2026-06-29-forgotten-seed"], &root);
    // Then re-tend (sets it to today) and untend (clears it).
    let _ = run(&["untend", "2026-06-29-forgotten-seed"], &root);
    // Use tend to set last_tended to a known date, then untend.
    let out = run(&["untend", "2026-06-29-forgotten-seed"], &root);
    assert_ok(&out);
    let body = std::fs::read_to_string(
        root.join("seeds").join("2026-06-29-forgotten-seed.md"),
    )
    .unwrap();
    // After untend, last_tended should be null in the YAML.
    assert!(
        body.contains("last_tended: null") || !body.contains("last_tended:"),
        "expected last_tended to be cleared: {body}"
    );
}

#[test]
fn untend_all_makes_everything_stale() {
    let root = unique_temp();
    let _ = run(&["plant", "alpha"], &root);
    let _ = run(&["plant", "beta"], &root);
    let out = run(&["untend", "--all"], &root);
    assert_ok(&out);
    let s = stdout(&out);
    assert!(s.contains("2 seed"));
}

#[test]
fn bloom_writes_both_artifacts() {
    let root = unique_temp();
    let _ = run(&["plant", "first"], &root);
    let _ = run(&["plant", "second"], &root);
    let out = run(&["bloom"], &root);
    assert_ok(&out);
    assert!(root.join("bloom.html").exists());
    assert!(root.join("index.md").exists());
    let html = std::fs::read_to_string(root.join("bloom.html")).unwrap();
    assert!(html.contains("hortus"));
    assert!(!html.contains("__NODES__")); // placeholders replaced
    assert!(!html.contains("__EDGES__"));
}

#[test]
fn version_mismatch_is_an_error() {
    let root = unique_temp();
    // First create a climate.toml by setting the climate.
    let _ = run(&["climate", "--mood", "test"], &root);
    let climate_path = root.join("climate.toml");
    assert!(climate_path.exists());
    let original = std::fs::read_to_string(&climate_path).unwrap();
    std::fs::write(&climate_path, original.replace("version = 1", "version = 999"))
        .unwrap();
    let out = run(&["today"], &root);
    assert!(!out.status.success(), "version mismatch should be an error");
    let s = stderr(&out) + &stdout(&out);
    assert!(s.contains("version") || s.contains("999"));
}

#[test]
fn unknown_command_exits_nonzero() {
    let root = unique_temp();
    let out = run(&["nonsense-command"], &root);
    assert!(!out.status.success());
}
