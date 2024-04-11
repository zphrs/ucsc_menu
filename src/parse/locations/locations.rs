use juniper::GraphQLObject;

use crate::{parse::Error, static_selector};

use super::location_meta::LocationMeta;

struct Locations {
    locations: Vec<LocationMeta>,
}

impl Locations {
    pub(super) fn from_html_element(element: scraper::ElementRef) -> Result<Self, Error> {
        static_selector!(LOCATION_CHOICES_SELECTOR <- "div#locationchoices");
        static_selector!(LOCATION_SELECTOR <- "li.locations");

        let Some(choices) = element.select(&LOCATION_CHOICES_SELECTOR).next() else {
            return Err(Error::html_parse_error(
                "Location choices element not found",
            ));
        };

        let location_matches = choices.select(&LOCATION_SELECTOR);
        let mut locations = Vec::with_capacity(location_matches.size_hint().0);
        for location in location_matches {
            let location_meta = LocationMeta::from_html_element(location)?;
            locations.push(location_meta);
        }

        Ok(Self { locations })
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
