use crate::parse::daily_menu::DailyMenu;
use crate::parse::Error;
struct Location<'a> {
    name: &'a str,
    id: &'a str, // ex. 40 for 9/10
    daily_menus: Vec<DailyMenu<'a>>,
}

impl<'a> Location<'a> {
    fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // TODO
        todo!("Implement the Location::from_html_element function")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_from_html_element() {
        let html = fs::read_to_string("./src/parse/html_examples/locations/location.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let location = Location::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(location.name, "College Nine/John R. Lewis Dining Hall");
        assert_eq!(location.id, "40");
        println!("{:?}", location.daily_menus);
    }
}
