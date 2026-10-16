use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{LazyLock, Mutex};

use gpui::SharedString;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-hans")]
    SimplifiedChinese,
    #[default]
    #[serde(rename = "system", other)]
    System,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::System, Self::English, Self::SimplifiedChinese];

    pub fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::English => "en",
            Self::SimplifiedChinese => "zh-hans",
        }
    }

    pub fn label(self) -> SharedString {
        self.label_source().resolve()
    }

    pub fn label_source(self) -> Text {
        match self {
            Self::System => message("language.system"),
            Self::English => "English".into(),
            Self::SimplifiedChinese => "简体中文".into(),
        }
    }

    pub fn resolve(self, system: Option<&str>) -> Locale {
        match self {
            Self::English => Locale::English,
            Self::SimplifiedChinese => Locale::SimplifiedChinese,
            Self::System => Locale::from_system(system),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Locale {
    English,
    SimplifiedChinese,
}

impl Locale {
    pub fn from_system(locale: Option<&str>) -> Self {
        if locale.is_some_and(|locale| {
            locale
                .split(['-', '_', '.'])
                .next()
                .is_some_and(|language| language.eq_ignore_ascii_case("zh"))
        }) {
            Self::SimplifiedChinese
        } else {
            Self::English
        }
    }
}

static ACTIVE_LOCALE: AtomicU8 = AtomicU8::new(Locale::English as u8);
type Catalog = BTreeMap<String, String>;
static ENGLISH: LazyLock<Catalog> = LazyLock::new(|| {
    toml::from_str(include_str!("../i18n/en.toml")).expect("valid embedded English catalog")
});
static CHINESE: LazyLock<Catalog> = LazyLock::new(|| {
    toml::from_str(include_str!("../i18n/zh-hans.toml")).expect("valid embedded Chinese catalog")
});
static MISSING: LazyLock<Mutex<BTreeSet<String>>> = LazyLock::new(Mutex::default);

pub fn active_locale() -> Locale {
    #[cfg(any(test, debug_assertions))]
    if let Some(locale) = testing::override_locale() {
        return locale;
    }
    match ACTIVE_LOCALE.load(Ordering::Relaxed) {
        1 => Locale::SimplifiedChinese,
        _ => Locale::English,
    }
}

pub fn set_active_locale(locale: Locale) {
    #[cfg(any(test, debug_assertions))]
    if testing::set_override_if_scoped(locale) {
        return;
    }
    ACTIVE_LOCALE.store(locale as u8, Ordering::Relaxed);
}

fn lookup<'a>(locale: Locale, key: &str, en: &'a Catalog, zh: &'a Catalog) -> Option<&'a str> {
    let catalog = match locale {
        Locale::English => en,
        Locale::SimplifiedChinese => zh,
    };
    catalog.get(key).or_else(|| en.get(key)).map(String::as_str)
}

fn first_missing(key: &str, warned: &Mutex<BTreeSet<String>>) -> bool {
    warned
        .lock()
        .expect("missing-key warnings lock")
        .insert(key.into())
}

fn lookup_active(key: &str) -> Option<&'static str> {
    let locale = active_locale();
    let catalog = if locale == Locale::English {
        &*ENGLISH
    } else {
        &*CHINESE
    };
    if !catalog.contains_key(key) && first_missing(key, &MISSING) {
        tracing::warn!(key, ?locale, "missing UI translation");
    }
    lookup(locale, key, &ENGLISH, &CHINESE)
}

pub fn t(key: &str) -> SharedString {
    lookup_active(key).map_or_else(|| key.to_owned().into(), SharedString::from)
}

pub fn text_for(locale: Locale, key: &'static str) -> &'static str {
    lookup(locale, key, &ENGLISH, &CHINESE).unwrap_or(key)
}

pub fn text(key: &'static str) -> &'static str {
    lookup_active(key).unwrap_or(key)
}

pub fn tf(key: &str, arguments: &[(&str, String)]) -> String {
    format_template(&t(key), arguments)
}

fn format_template(template: &str, arguments: &[(&str, String)]) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find('}') else { break };
        let name = &rest[1..end];
        if let Some((_, value)) = arguments.iter().find(|(key, _)| *key == name) {
            result.push_str(value);
        } else {
            result.push_str(&rest[..=end]);
        }
        rest = &rest[end + 1..];
    }
    result.push_str(rest);
    result
}

pub fn tn(key: &str, count: usize) -> String {
    tnf(key, count, &[])
}

fn plural_template<'a>(
    locale: Locale,
    key: &str,
    count: usize,
    en: &'a Catalog,
    zh: &'a Catalog,
) -> Option<&'a str> {
    let english = format!("{key}.{}", if count == 1 { "one" } else { "other" });
    let translated = (locale == Locale::SimplifiedChinese)
        .then(|| zh.get(&format!("{key}.other")))
        .flatten();
    translated.or_else(|| en.get(&english)).map(String::as_str)
}

pub fn tnf(key: &str, count: usize, arguments: &[(&str, String)]) -> String {
    let locale = active_locale();
    let form = if locale == Locale::English && count == 1 {
        "one"
    } else {
        "other"
    };
    let requested = format!("{key}.{form}");
    let catalog = if locale == Locale::English {
        &*ENGLISH
    } else {
        &*CHINESE
    };
    if !catalog.contains_key(&requested) && first_missing(&requested, &MISSING) {
        tracing::warn!(key = requested, ?locale, "missing UI translation");
    }
    let mut values = vec![("count", count.to_string())];
    values.extend_from_slice(arguments);
    format_template(
        plural_template(locale, key, count, &ENGLISH, &CHINESE).unwrap_or(&requested),
        &values,
    )
}

/// Cached UI copy keeps its key; literal user and backend text never enters a catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Text {
    Literal(SharedString),
    Label {
        key: &'static str,
        english: SharedString,
    },
    Catalog {
        key: &'static str,
        arguments: Vec<(&'static str, Text)>,
        count: Option<usize>,
    },
}

pub fn localized_label(key: &'static str, english: &str) -> Text {
    Text::Label {
        key,
        english: english.to_owned().into(),
    }
}

pub fn message(key: &'static str) -> Text {
    messagef(key, &[])
}

pub fn messagef(key: &'static str, arguments: &[(&'static str, String)]) -> Text {
    message_with(
        key,
        &arguments
            .iter()
            .map(|(key, value)| (*key, value.clone().into()))
            .collect::<Vec<_>>(),
    )
}

pub fn message_with(key: &'static str, arguments: &[(&'static str, Text)]) -> Text {
    Text::Catalog {
        key,
        arguments: arguments.to_vec(),
        count: None,
    }
}

pub fn message_count(
    key: &'static str,
    count: usize,
    arguments: &[(&'static str, String)],
) -> Text {
    Text::Catalog {
        key,
        arguments: arguments
            .iter()
            .map(|(key, value)| (*key, value.clone().into()))
            .collect(),
        count: Some(count),
    }
}

impl Text {
    pub fn identity(&self) -> &str {
        match self {
            Self::Literal(text) => text,
            Self::Label { key, .. } | Self::Catalog { key, .. } => key,
        }
    }

    pub fn resolve(&self) -> SharedString {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Label { key, english } => {
                if active_locale() == Locale::English {
                    english.clone()
                } else {
                    t(key)
                }
            }
            Self::Catalog {
                key,
                arguments,
                count,
            } => {
                if arguments.is_empty() && count.is_none() {
                    return t(key);
                }
                let arguments = arguments
                    .iter()
                    .map(|(key, value)| (*key, value.resolve().to_string()))
                    .collect::<Vec<_>>();
                match count {
                    Some(count) => tnf(key, *count, &arguments).into(),
                    None => tf(key, &arguments).into(),
                }
            }
        }
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::Literal(text.to_owned().into())
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::Literal(text.into())
    }
}

impl From<SharedString> for Text {
    fn from(text: SharedString) -> Self {
        Self::Literal(text)
    }
}

#[cfg(any(test, debug_assertions))]
pub mod testing {
    use super::Locale;
    use std::cell::Cell;

    thread_local! { static OVERRIDE: Cell<Option<Locale>> = const { Cell::new(None) }; }

    pub(super) fn override_locale() -> Option<Locale> {
        OVERRIDE.get()
    }

    pub(super) fn set_override_if_scoped(locale: Locale) -> bool {
        if OVERRIDE.get().is_some() {
            OVERRIDE.set(Some(locale));
            true
        } else {
            false
        }
    }

    pub fn with_locale<R>(locale: Locale, run: impl FnOnce() -> R) -> R {
        struct Restore(Option<Locale>);
        impl Drop for Restore {
            fn drop(&mut self) {
                OVERRIDE.set(self.0);
            }
        }
        let _restore = Restore(OVERRIDE.replace(Some(locale)));
        run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(value: &str) -> BTreeSet<&str> {
        value
            .split('{')
            .skip(1)
            .filter_map(|part| part.split_once('}').map(|(key, _)| key))
            .collect()
    }

    #[test]
    fn catalogs_have_the_same_keys_placeholders_and_valid_dates() {
        assert_eq!(
            ENGLISH.keys().collect::<Vec<_>>(),
            CHINESE.keys().collect::<Vec<_>>()
        );
        for (key, english) in &*ENGLISH {
            let chinese = &CHINESE[key];
            assert!(
                !english.is_empty() || key.ends_with(".description"),
                "{key}"
            );
            assert!(!chinese.is_empty() || english.is_empty(), "{key}");
            assert_eq!(placeholders(english), placeholders(chinese), "{key}");
            if key.starts_with("date.") {
                for pattern in [english, chinese] {
                    assert!(
                        !chrono::format::StrftimeItems::new(pattern)
                            .any(|item| item == chrono::format::Item::Error),
                        "{key}: {pattern}"
                    );
                }
            }
        }
    }

    #[test]
    fn substitution_is_unicode_safe_single_pass_and_leaves_missing_arguments_visible() {
        assert_eq!(
            format_template(
                "你好 {name}，{count} 次 {missing}",
                &[("name", "{count} 🦀".into()), ("count", "2".into())]
            ),
            "你好 {count} 🦀，2 次 {missing}"
        );
        assert_eq!(
            format_template("literal {unclosed", &[]),
            "literal {unclosed"
        );
    }

    #[test]
    fn fallback_and_warnings_are_independent_of_catalog_completeness() {
        let en = BTreeMap::from([("only-en".into(), "English fallback".into())]);
        assert_eq!(
            lookup(Locale::SimplifiedChinese, "only-en", &en, &BTreeMap::new()),
            Some("English fallback")
        );
        assert_eq!(
            lookup(Locale::English, "unknown", &en, &BTreeMap::new()),
            None
        );
        let warned = Mutex::default();
        assert!(first_missing("unknown", &warned));
        assert!(!first_missing("unknown", &warned));
        assert!(first_missing("another", &warned));
        assert_eq!(
            t("test.key-that-does-not-exist"),
            "test.key-that-does-not-exist"
        );
        let plurals = BTreeMap::from([
            ("items.one".into(), "{count} item".into()),
            ("items.other".into(), "{count} items".into()),
        ]);
        assert_eq!(
            plural_template(
                Locale::SimplifiedChinese,
                "items",
                1,
                &plurals,
                &BTreeMap::new()
            ),
            Some("{count} item")
        );
        assert_eq!(
            plural_template(
                Locale::SimplifiedChinese,
                "items",
                2,
                &plurals,
                &BTreeMap::new()
            ),
            Some("{count} items")
        );
    }

    #[test]
    fn system_resolution_and_intent_are_separate() {
        for tag in ["zh", "zh-CN", "zh-Hans", "ZH_tw", "zh-Hant-HK"] {
            assert_eq!(
                Language::System.resolve(Some(tag)),
                Locale::SimplifiedChinese
            );
        }
        for tag in [None, Some("en-CN"), Some("de-DE"), Some(""), Some("zho")] {
            assert_eq!(Language::System.resolve(tag), Locale::English);
        }
        assert_eq!(Language::English.resolve(Some("zh-CN")), Locale::English);
        assert_eq!(
            Language::SimplifiedChinese.resolve(None),
            Locale::SimplifiedChinese
        );
    }

    #[test]
    fn plurals_dates_and_nested_cached_arguments_follow_the_locale() {
        let cached = message_with(
            "format.runtime_default",
            &[("effort", message("effort.high"))],
        );
        let date = chrono::NaiveDate::from_ymd_opt(2026, 10, 16)
            .unwrap()
            .and_hms_opt(13, 45, 0)
            .unwrap();
        testing::with_locale(Locale::English, || {
            assert_eq!(tn("count.sessions", 0), "0 sessions");
            assert_eq!(tn("count.sessions", 1), "1 session");
            assert_eq!(tn("count.sessions", 2), "2 sessions");
            assert_eq!(
                date.format(text("date.checked")).to_string(),
                "Oct 16, 2026 at 1:45 PM"
            );
            assert_eq!(cached.resolve(), "Runtime default (High)");
        });
        testing::with_locale(Locale::SimplifiedChinese, || {
            assert_eq!(tn("count.sessions", 0), "0 个会话");
            assert_eq!(tn("count.sessions", 1), "1 个会话");
            assert_eq!(tn("count.sessions", 2), "2 个会话");
            assert_eq!(
                date.format(text("date.checked")).to_string(),
                "2026年10月16日 13:45"
            );
            assert_eq!(cached.resolve(), "运行时默认（高）");
            assert_eq!(Language::System.label(), "跟随系统");
            assert_eq!(Language::English.label(), "English");
            assert_eq!(Language::SimplifiedChinese.label(), "简体中文");
        });
    }

    #[test]
    fn cached_copy_re_resolves_and_scoped_locales_restore_on_unwind() {
        let cached = messagef("format.deleted_crew", &[("name", "{literal} 王".into())]);
        testing::with_locale(Locale::English, || {
            assert_eq!(cached.resolve(), "Deleted crew \"{literal} 王\".");
            let _ = std::panic::catch_unwind(|| {
                testing::with_locale(Locale::SimplifiedChinese, || {
                    assert_eq!(cached.resolve(), "已删除小队“{literal} 王”。");
                    panic!("restore");
                })
            });
            assert_eq!(cached.resolve(), "Deleted crew \"{literal} 王\".");
        });
    }
}
