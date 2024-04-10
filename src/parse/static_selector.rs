use std::sync::OnceLock;

use scraper::Selector;

#[derive(Debug)]
pub(super) struct StaticSelector<'a> {
    cell: OnceLock<Selector>,
    selector: &'a str,
}

impl<'a> StaticSelector<'a> {
    pub(super) const fn new(selector: &'a str) -> Self {
        Self {
            cell: OnceLock::new(),
            selector,
        }
    }
}

impl<'a> core::ops::Deref for StaticSelector<'a> {
    type Target = Selector;

    fn deref(&self) -> &Self::Target {
        self.cell
            .get_or_init(|| match Selector::parse(self.selector) {
                Ok(sel) => sel,
                Err(e) => panic!("Error parsing static selector {}: {:?}", self.selector, e),
            })
    }
}

#[macro_export]
macro_rules! static_selector {
    ($x: ident <- $sel: literal) => {
        static $x: $crate::parse::static_selector::StaticSelector =
            $crate::parse::static_selector::StaticSelector::new($sel);
    };
}
