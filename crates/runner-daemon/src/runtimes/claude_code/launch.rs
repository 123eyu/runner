use std::sync::Mutex;
use std::time::{Duration, Instant};

pub(crate) fn permission_arg(args: &[String]) -> String {
    args.windows(2)
        .find(|pair| pair[0] == "--permission-mode")
        .map(|pair| pair[1].clone())
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix("--permission-mode=").map(str::to_owned))
        })
        .unwrap_or_else(|| "default".into())
}

pub(crate) fn compute_gate_wait(last: Option<Instant>, now: Instant, grace: Duration) -> Duration {
    match last {
        None => Duration::ZERO,
        Some(t) => {
            let elapsed = now.saturating_duration_since(t);
            grace.saturating_sub(elapsed)
        }
    }
}

pub(crate) fn enter_gate(session_id: &str, state: &Mutex<Option<Instant>>, grace: Duration) {
    let mut last = state.lock().expect("launch_gate poisoned");
    let wait = compute_gate_wait(*last, Instant::now(), grace);
    if !wait.is_zero() {
        log::info!(
            "claude-code launch gate: session={session_id} sleep_ms={}",
            wait.as_millis()
        );
        std::thread::sleep(wait);
    }
    *last = Some(Instant::now());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_arg_preserves_native_values_and_split_flag_precedence() {
        for (args, expected) in [
            (vec![], "default"),
            (vec!["--permission-mode"], "default"),
            (vec!["--permission-mode=plan"], "plan"),
            (vec!["--permission-mode", "acceptEdits"], "acceptEdits"),
            (
                vec![
                    "--permission-mode=plan",
                    "--permission-mode",
                    "bypassPermissions",
                ],
                "bypassPermissions",
            ),
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert_eq!(permission_arg(&args), expected);
        }
    }
}
