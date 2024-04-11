use chrono::NaiveDate;

use crate::parse::LocationMeta;

struct menu_cache {
    date: NaiveDate,
    locations: Vec<LocationMeta>,
}
impl menu_cache {}
