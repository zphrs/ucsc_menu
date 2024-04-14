use chrono::NaiveDate;

use crate::parse::LocationMeta;

struct MenuCache {
    date: NaiveDate,
    locations: Vec<LocationMeta>,
}
impl MenuCache {
    fn new() {
        // TODO
        todo!("Implement the MenuCache::new function")
    }
    fn from_html_element(element: scraper::ElementRef) {
        // TODO
        todo!("Implement the MenuCache::from_html_element function")
    }
    fn handle_graphql_request(&self) {
        // TODO
        todo!("Implement the MenuCache::handle_graphql_request function")
    }
}
