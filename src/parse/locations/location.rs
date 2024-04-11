use std::borrow::{Borrow, Cow};
use std::sync::Arc;

use crate::parse::daily_menu::DailyMenu;
use crate::parse::Error;
use crate::static_selector;
use scraper::Html;
use url::Url;

#[derive(Debug)]
pub(super) struct Location<'a> {
    name: Cow<'a, str>,
    id: String, // ex. 40 for 9/10
    daily_menus: Vec<DailyMenu<'a>>,
    html: Arc<Html>,
}

impl<'a> Location<'a> {
    pub(super) async fn from_html_element(
        client: &reqwest::Client,
        element: scraper::ElementRef<'a>,
    ) -> Result<Self, Error> {
        static_selector!(LOCATION_SELECTOR <- ".locations > a");
        let Some(location_element) = element.select(&LOCATION_SELECTOR).next() else {
            return Err(Error::html_parse_error("location name node not found"));
        };

        // TODO: make static
        let url = Url::parse("https://nutrition.sa.ucsc.edu").expect("base url should be valid!");
        let Ok(url) =
            url.join(location_element.attr("href").ok_or_else(|| {
                Error::html_parse_error("location <a> does not have a href attr")
            })?)
        else {
            return Err(Error::html_parse_error("Location url is invalid"));
        };

        let id = url
            .query_pairs()
            .find(|x| x.0 == "locationNum")
            .ok_or_else(|| {
                Error::html_parse_error(
                    "Location url does not include the `locationNum` query parameter",
                )
            })?
            .1
            .into_owned();

        // there is whitespace sometimes - TODO check if we can always just take the second element
        let mut name = element.text().collect::<Cow<'a, str>>();

        if name.trim() != name {
            name = Cow::Owned(name.trim().to_owned());
        }

        if name.is_empty() {
            return Err(Error::html_parse_error("Location name is missing"));
        }
        let mut cookies = String::with_capacity(100);
        cookies.push_str("WebInaCartDates=; ");
        cookies.push_str("WebInaCartLocation=");
        cookies.push_str(&id);
        cookies.push_str("; WebInaCartMeals=; WebInaCartQtys=; WebInaCartRecipes=");
        let menu = client
            .get(url)
            .header("Cookie", cookies)
            .send()
            .await?
            .text()
            .await?;

        let html = Html::parse_document(&menu);

        let html = Arc::new(html);

        let out = Self {
            name,
            id,
            daily_menus: vec![],
            html,
        };

        let daily_menu = DailyMenu::from_html_element(out.html.root_element())?;
        let daily_menus = vec![daily_menu];

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_from_html_element() {
        let html = fs::read_to_string("./src/parse/html_examples/locations/location.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let client = reqwest::Client::builder()
            // TODO - my machine doesn't like the nutrition.sa cert on linux - i need to figure
            // out if that's their issue or mine
            .danger_accept_invalid_certs(true)
            .build()
            .expect("error building client");
        let location = Location::from_html_element(&client, document.root_element())
            .await
            .expect("The example html should be valid");
        assert_eq!(location.name, "College Nine/John R. Lewis Dining Hall");
        assert_eq!(location.id, "40");
        println!("{:?}", location.daily_menus);
    }
}
