use crate::parse::Error;
use std::panic::Location;
struct Locations<'a> {
    locations: Vec<Location<'a>>,
}

impl<'a> Locations<'a> {
    fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // TODO
        todo!("Implement the Locations::from_html_element function")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_from_html_element() {
        let html =
            fs::read_to_string("./src/parse/html_examples/locations/locations.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let locations = Locations::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(locations.locations.len(), 14);
        println!("{:?}", locations.locations);
    }
}
