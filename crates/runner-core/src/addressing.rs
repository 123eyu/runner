use crate::model::Recipients;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub recipients: Recipients,
    pub text: String,
}

pub fn resolve(text: &str, explicit: &[String], roster: &[String]) -> crate::Result<Resolved> {
    let targets: Vec<String> = if explicit.is_empty() {
        text.split_whitespace()
            .map_while(mention_handle)
            .map(str::to_owned)
            .collect()
    } else {
        explicit.to_vec()
    };
    for target in &targets {
        if target == "human" || !roster.contains(target) {
            return Err(crate::Error::msg(format!(
                "@{target} is not a message recipient; valid handles: {}",
                roster
                    .iter()
                    .map(|handle| format!("@{handle}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }
    Ok(Resolved {
        recipients: Recipients::new(targets),
        text: text.to_owned(),
    })
}

fn mention_handle(token: &str) -> Option<&str> {
    let handle = token.strip_prefix('@')?;
    let handle = handle
        .strip_suffix(':')
        .or_else(|| handle.strip_suffix(','))
        .unwrap_or(handle);
    let mut bytes = handle.bytes();
    let first = bytes.next()?;
    ((first.is_ascii_lowercase() || first.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        }))
    .then_some(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_only_leading_mentions_and_explicit_recipients_win() {
        let roster = ["coder", "qa", "reviewer", "impl", "1_coder-qa"]
            .map(str::to_owned)
            .to_vec();
        for (text, explicit, expected) in [
            ("hello @qa", vec![], vec![]),
            ("@coder hello", vec![], vec!["coder"]),
            ("@coder: please check", vec![], vec!["coder"]),
            ("@coder, please check", vec![], vec!["coder"]),
            ("@coder, @qa: hi", vec![], vec!["coder", "qa"]),
            ("@1_coder-qa: hi", vec![], vec!["1_coder-qa"]),
            ("@impl's fix", vec![], vec![]),
            ("@reviewer's audit @qa", vec![], vec![]),
            ("@coder @impl's fix @qa", vec![], vec!["coder"]),
            ("@", vec![], vec![]),
            ("@ @qa", vec![], vec![]),
            ("@coder:: @qa", vec![], vec![]),
            ("@coder,, @qa", vec![], vec![]),
            ("@Coder @qa", vec![], vec![]),
            ("@_coder @qa", vec![], vec![]),
            (" @coder @qa hello", vec![], vec!["coder", "qa"]),
            ("@human hello", vec!["qa"], vec!["qa"]),
            (
                "@missing hello",
                vec!["qa", "qa", "reviewer"],
                vec!["qa", "reviewer"],
            ),
        ] {
            let explicit: Vec<String> = explicit.into_iter().map(str::to_owned).collect();
            let result = resolve(text, &explicit, &roster).unwrap();
            assert_eq!(result.recipients.as_slice(), expected);
            assert_eq!(result.text, text);
        }
        for (text, explicit) in [
            ("hello", vec!["human".into()]),
            ("@human hello", vec![]),
            ("@human: hello", vec![]),
            ("@human, hello", vec![]),
            ("@missing hello", vec![]),
            ("@missing: hello", vec![]),
            ("@missing, hello", vec![]),
        ] {
            let error = resolve(text, &explicit, &roster).unwrap_err().to_string();
            for handle in &roster {
                assert!(error.contains(handle));
            }
        }
    }
}
