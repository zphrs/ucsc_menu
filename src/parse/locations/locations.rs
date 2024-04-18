use juniper::GraphQLObject;
use scraper::Html;

use crate::{parse::Error, static_selector};

use super::location_meta::LocationMeta;

use super::location_data::LocationData;

#[derive(Debug)]
pub struct Location<'a>(LocationData<'a>, LocationMeta);

impl<'a> Location<'a> {
    pub fn new(location_meta: LocationMeta) -> Self {
        Self(LocationData::new(), location_meta)
    }

    pub fn add_meals(&mut self, html: Vec<&'a Html>) -> Result<(), Error> {
        // TODO: instead of immediately clearing, diff the similar meals first
        self.clear();
        for html in html {
            self.0.add_meal(html)?;
        }
        Ok(())
    }

    pub fn hydrated(&self) -> bool {
        !self.0.empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

pub struct Locations<'a> {
    locations: Vec<Location<'a>>,
}

impl<'a> Locations<'a> {
    pub fn from_html_element(element: scraper::ElementRef) -> Result<Self, Error> {
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
            locations.push(Location::new(location_meta));
        }

        Ok(Self { locations })
    }

    pub fn add_meals<'b: 'a>(
        &mut self,
        html: Vec<&'b Html>,
        location_meta: LocationMeta,
    ) -> Result<(), Error> {
        let location = self
            .locations
            .iter_mut()
            .find(|x| x.1 == location_meta)
            .ok_or_else(|| {
                Error::InternalError(format!(
                    "Location with id {} is either already hydrated or does not exist. Clear all locations and try again.",
                    location_meta.id()
                ))
            })?;

        location.add_meals(html)?;

        Ok(())
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
