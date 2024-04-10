use futures::future;

use crate::{parse::Error, static_selector};

use super::location::Location;
struct Locations<'a> {
    locations: Vec<Location<'a>>,
}

impl<'a> Locations<'a> {
    pub(super) async fn from_html_element(
        client: &reqwest::Client,
        element: scraper::ElementRef<'a>,
    ) -> Result<Self, Error> {
        static_selector!(LOCATION_CHOICES_SELECTOR <- "div#locationchoices");
        static_selector!(LOCATION_SELECTOR <- "li.locations");

        let Some(choices) = element.select(&LOCATION_CHOICES_SELECTOR).next() else {
            return Err(Error::html_parse_error(
                "Location choices element not found",
            ));
        };

        let locations = future::try_join_all(
            choices
                .select(&LOCATION_SELECTOR)
                .map(|x| Location::from_html_element(client, x)),
        )
        .await?;

        Ok(Self { locations })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_from_html_element() {
        let html =
            fs::read_to_string("./src/parse/html_examples/locations/locations.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let client = reqwest::Client::builder()
            // TODO - my machine doesn't like the nutrition.sa cert on linux - i need to figure
            // out if that's their issue or mine
            .danger_accept_invalid_certs(true)
            .build()
            .expect("error building client");
        let locations = Locations::from_html_element(&client, document.root_element())
            .await
            .expect("The example html should be valid");
        assert_eq!(locations.locations.len(), 14);
        println!("{:?}", locations.locations);
    }
}
