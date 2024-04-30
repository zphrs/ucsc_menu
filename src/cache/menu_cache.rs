use chrono::{DateTime, NaiveDate, Utc};

use crate::parse::Locations;

struct MenuCache<'a> {
    cached_at: DateTime<Utc>,
    locations: Locations<'a>,
}

impl<'a> MenuCache<'a> {
    pub fn open() {
        // get from db
    }

    fn fetch_from_db() -> Self {
        // get from db
        todo!()
    }

    pub fn save() {}

    pub fn refresh(&self) {
        // if last_cached is younger than 15 minutes, return
    }
}
