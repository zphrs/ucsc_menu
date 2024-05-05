use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;

pub fn remove_excess_whitespace<'a>(s: &'a str) -> Cow<'a, str> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\s\s+").expect("regex should be valid"));
    let out: Cow<'a, str> = Regex::replace_all(re, s, " ");
    out
}
