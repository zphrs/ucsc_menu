use chrono::NaiveDate;

use crate::parse::Locations;

struct MenuCache<'a> {
    date: NaiveDate,
    locations: Locations<'a>,
}

impl<'a> MenuCache<'a> {}
