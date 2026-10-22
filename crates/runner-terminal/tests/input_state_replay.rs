use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use runner_terminal::fixtures::{decode_chunk, Fixture, FixtureEvent, FixtureInput};
use runner_terminal::input_state::{InputEvent, InputObservation, InputState, InputTracker};
use runner_terminal::mappings::InputKind;
use runner_terminal::replay::{feed, new_term};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn transition(ms: u64, observation: InputObservation) -> String {
    format!(
        "{ms} {:?} composing={} visible={}",
        observation.state, observation.composing, observation.composer_visible
    )
}

#[test]
fn native_submit_before_echo_releases_at_a_new_empty_composer() {
    for (cols, empty) in [
        (40, "Ask Codex to do anything"),
        (120, "Ask Codex to do anything"),
        (120, "Review this repository"),
        (120, ""),
    ] {
        for pasted in [false, true] {
            for delayed_empty in [false, true] {
                submitted_history_releases_at_a_new_empty_composer(
                    false,
                    cols,
                    pasted,
                    delayed_empty,
                    empty,
                );
            }
        }
    }
}

#[test]
fn native_submit_after_echo_releases_at_a_new_empty_composer() {
    for (cols, empty) in [
        (40, "Ask Codex to do anything"),
        (120, "Ask Codex to do anything"),
        (120, "Review this repository"),
        (120, ""),
    ] {
        for pasted in [false, true] {
            for delayed_empty in [false, true] {
                submitted_history_releases_at_a_new_empty_composer(
                    true,
                    cols,
                    pasted,
                    delayed_empty,
                    empty,
                );
            }
        }
    }
}

fn submitted_history_releases_at_a_new_empty_composer(
    echo_before_submit: bool,
    cols: u16,
    pasted: bool,
    delayed_empty: bool,
    empty: &str,
) {
    let start = Instant::now();
    let mut term = new_term(cols, 10);
    feed(
        &mut term,
        "\x1b[2;1H› \x1b[90mAsk Codex to do anything\x1b[39m\x1b[2;3H".as_bytes(),
    );
    let mut tracker = InputTracker::new(start);
    let mut state = tracker.initial_observation(start).state;
    let text =
        "Run python3 /private/tmp/runner-940.fixture/qa_worker.py read once. Then ACK QA940.";
    if pasted {
        tracker.observe_input(&InputEvent::Paste { text: text.into() }, start, &term);
    } else {
        for character in text.chars() {
            tracker.observe_input(
                &InputEvent::Key {
                    kind: InputKind::Content {
                        text: character.to_string(),
                    },
                },
                start,
                &term,
            );
        }
    }
    if echo_before_submit {
        feed(&mut term, format!("\x1b[2;1H\x1b[K› {text}").as_bytes());
        state = tracker.observe_output(start, &term).unwrap().state;
        assert_eq!(state, InputState::Drafting);
    }
    if let Some(observation) = tracker.observe_input(
        &InputEvent::Key {
            kind: InputKind::Submit,
        },
        start,
        &term,
    ) {
        state = observation.state;
    }
    if delayed_empty {
        feed(
            &mut term,
            format!("\x1b[2;1H\x1b[K› {text}\x1b[4;1HACK QA940").as_bytes(),
        );
        if let Some(observation) = tracker.observe_output(start + Duration::from_millis(1), &term) {
            state = observation.state;
        }
        assert_eq!(state, InputState::Drafting);
    }
    feed(
        &mut term,
        format!(
            "\x1b[2;1H\x1b[K› {text}\x1b[4;1HACK QA940\x1b[7;1H› \x1b[90m{empty}\x1b[39m\x1b[7;3H"
        )
        .as_bytes(),
    );
    if let Some(observation) = tracker.observe_output(start + Duration::from_millis(30), &term) {
        state = observation.state;
    }
    assert_eq!(state, InputState::Idle);
    assert!(tracker
        .observe_output(start + Duration::from_secs(30), &term)
        .is_none());

    tracker.observe_input(
        &InputEvent::Key {
            kind: InputKind::Content {
                text: "unsent".into(),
            },
        },
        start + Duration::from_secs(31),
        &term,
    );
    feed(&mut term, "\x1b[7;1H\x1b[K› unsent".as_bytes());
    assert_eq!(
        tracker
            .observe_output(start + Duration::from_secs(31), &term)
            .unwrap()
            .state,
        InputState::Drafting
    );
    feed(&mut term, b"\x1b[3S\x1b[4;9H");
    assert!(tracker
        .observe_output(start + Duration::from_secs(60), &term)
        .is_none());
    feed(
        &mut term,
        format!("\x1b[4;1H\x1b[K› \x1b[90m{empty}\x1b[39m\x1b[4;3H").as_bytes(),
    );
    assert_eq!(
        tracker
            .observe_output(start + Duration::from_secs(61), &term)
            .unwrap()
            .state,
        InputState::Idle
    );
}

#[test]
fn a_rejected_native_probe_submit_holds_on_its_only_redraw() {
    for reprobe in [false, true] {
        let start = Instant::now();
        let mut term = new_term(80, 10);
        feed(&mut term, "\x1b[2;1H› Ask Codex".as_bytes());
        let mut tracker = InputTracker::new(start);
        tracker.initial_observation(start);
        let submit = InputEvent::Key {
            kind: InputKind::Submit,
        };
        if reprobe {
            tracker.observe_input(&InputEvent::Paste { text: "old".into() }, start, &term);
            feed(&mut term, "\x1b[2;1H\x1b[K› old".as_bytes());
            tracker.observe_output(start, &term);
            tracker.observe_input(&submit, start, &term);
        }
        tracker.observe_input(
            &InputEvent::Paste {
                text: "line one\nline two".into(),
            },
            start,
            &term,
        );
        tracker.observe_input(&submit, start, &term);
        feed(
            &mut term,
            "\x1b[2;1H\x1b[K› line one\x1b[3;1H  line two\x1b[4;3H".as_bytes(),
        );
        let observation = tracker
            .observe_output(start + Duration::from_millis(1), &term)
            .unwrap();
        assert_eq!(observation.state, InputState::Drafting, "reprobe={reprobe}");
    }
}

#[test]
fn native_submit_during_reprobe_releases_at_a_new_empty_composer() {
    let start = Instant::now();
    let mut term = new_term(80, 10);
    feed(&mut term, "\x1b[2;1H› Ask Codex".as_bytes());
    let mut tracker = InputTracker::new(start);
    tracker.initial_observation(start);
    tracker.observe_input(&InputEvent::Paste { text: "old".into() }, start, &term);
    feed(&mut term, "\x1b[2;1H\x1b[K› old".as_bytes());
    tracker.observe_output(start, &term);
    let submit = InputEvent::Key {
        kind: InputKind::Submit,
    };
    tracker.observe_input(&submit, start, &term);
    tracker.observe_input(&InputEvent::Paste { text: "new".into() }, start, &term);
    tracker.observe_input(&submit, start, &term);
    feed(
        &mut term,
        "\x1b[2J\x1b[1;1H› old\x1b[4;1H› new\x1b[7;1H› Ask Codex\x1b[7;3H".as_bytes(),
    );
    assert_eq!(
        tracker
            .observe_output(start + Duration::from_millis(1), &term)
            .unwrap()
            .state,
        InputState::Idle
    );
}

#[test]
fn a_rejected_multiline_submit_holds_with_the_cursor_on_a_blank_continuation() {
    let start = Instant::now();
    let mut term = new_term(80, 10);
    feed(&mut term, "\x1b[2;1H› Ask Codex".as_bytes());
    let mut tracker = InputTracker::new(start);
    tracker.initial_observation(start);
    tracker.observe_input(
        &InputEvent::Paste {
            text: "line one\nline two".into(),
        },
        start,
        &term,
    );
    feed(
        &mut term,
        "\x1b[2;1H\x1b[K› line one\x1b[3;1H  line two".as_bytes(),
    );
    tracker.observe_output(start, &term);
    tracker.observe_input(
        &InputEvent::Key {
            kind: InputKind::Submit,
        },
        start,
        &term,
    );
    feed(&mut term, b"\x1b[4;3H");
    assert_eq!(
        tracker
            .observe_output(start + Duration::from_millis(1), &term)
            .unwrap()
            .state,
        InputState::Drafting
    );
    assert!(tracker
        .observe_output(start + Duration::from_secs(30), &term)
        .is_none());
}

#[test]
fn a_composition_return_cannot_clear_a_draft_using_another_empty_prompt() {
    let start = Instant::now();
    let mut term = new_term(80, 10);
    feed(
        &mut term,
        "\x1b[2;1H› Ask Codex\x1b[7;1H› Ask Codex\x1b[2;3H".as_bytes(),
    );
    let mut tracker = InputTracker::new(start);
    tracker.initial_observation(start);
    tracker.observe_input(
        &InputEvent::Paste {
            text: "line one\nline two".into(),
        },
        start,
        &term,
    );
    tracker.observe_input(&InputEvent::Composing { composing: true }, start, &term);
    tracker.observe_input(
        &InputEvent::Key {
            kind: InputKind::Submit,
        },
        start,
        &term,
    );
    feed(
        &mut term,
        "\x1b[2;1H\x1b[K› line one\x1b[3;1H  line two\x1b[7;3H".as_bytes(),
    );
    tracker.observe_output(start + Duration::from_millis(1), &term);
    let observation = tracker
        .observe_input(
            &InputEvent::Composing { composing: false },
            start + Duration::from_millis(2),
            &term,
        )
        .unwrap();
    assert_eq!(observation.state, InputState::Drafting);
    assert!(tracker
        .observe_output(start + Duration::from_secs(30), &term)
        .is_none());
}

#[test]
fn recorded_input_transitions_follow_the_grid() {
    let mut paths = std::fs::read_dir(fixtures_dir())
        .expect("fixtures dir")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("input-") && name.ends_with(".ndjson"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    assert!(!paths.is_empty(), "input fixture corpus is empty");

    for path in paths {
        let fixture = Fixture::load(&path).unwrap_or_else(|error| {
            panic!("load {}: {error:#}", path.display());
        });
        let started = Instant::now();
        let mut term = new_term(fixture.header.cols, fixture.header.rows);
        let mut tracker = InputTracker::new(started);
        let mut actual = vec![transition(0, tracker.initial_observation(started))];
        for event in fixture.events {
            let (ms, observation) = match event {
                FixtureEvent::Data { ms, data } => {
                    feed(&mut term, &decode_chunk(&data).unwrap());
                    (
                        ms,
                        tracker.observe_output(started + Duration::from_millis(ms), &term),
                    )
                }
                FixtureEvent::Input {
                    ms,
                    input: FixtureInput::Event(input),
                } => (
                    ms,
                    tracker.observe_input(&input, started + Duration::from_millis(ms), &term),
                ),
                FixtureEvent::Input {
                    input: FixtureInput::LegacyBytes(_),
                    ..
                }
                | FixtureEvent::Exit { .. } => continue,
                FixtureEvent::Resize { cols, rows, .. } => {
                    term.resize(alacritty_terminal::term::test::TermSize::new(
                        cols as usize,
                        rows as usize,
                    ));
                    continue;
                }
            };
            if let Some(observation) = observation {
                actual.push(transition(ms, observation));
            }
        }
        let actual = format!("{}\n", actual.join("\n"));
        let expected_path = path.with_extension("expected.txt");
        let expected = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("missing {}", expected_path.display()));
        assert_eq!(actual, expected, "transition drift in {}", path.display());
    }
}
