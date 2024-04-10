#[macro_export]
macro_rules! get_or_init_selector {
    ($sel:expr,$query:literal) => {
        $sel.get_or_init(|| Selector::parse($query)
        .expect("Selector is valid")) // I use expect here because all queries are hardcoded and therefore should always be valid
    };
}
