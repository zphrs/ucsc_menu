use crate::parse::Error;
use crate::static_selector;
use url::Url;
#[derive(Debug, Clone, PartialEq)]
pub struct LocationMeta {
    name: String,
    id: String, // ex. 40 for 9/10
    url: Url,
}

impl serde::Serialize for LocationMeta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.url().as_str().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for LocationMeta {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        let url = Url::parse(&str).map_err(serde::de::Error::custom)?;
        Self::from_url(url).map_err(serde::de::Error::custom)
    }
}

impl Eq for LocationMeta {}

impl LocationMeta {
    pub const fn url(&self) -> &Url {
        &self.url
    }

    pub fn from_url(url: Url) -> Result<Self, Error> {
        let mut query_pairs = url.query_pairs();

        let id = query_pairs
            .find(|x| x.0 == "locationNum")
            .ok_or_else(|| {
                Error::html_parse_error(
                    "Location url does not include the `locationNum` query parameter",
                )
            })?
            .1
            .into_owned();

        let name = query_pairs
            .find(|x| x.0 == "locationName")
            .ok_or_else(|| {
                Error::html_parse_error(
                    "Location url does not include the `locationName` query parameter",
                )
            })?
            .1
            .into_owned();

        Ok(Self { name, id, url })
    }

    pub(super) fn from_html_element(element: scraper::ElementRef) -> Result<Self, Error> {
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

        Self::from_url(url)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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
        let location = LocationMeta::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(location.name, "College Nine/John R. Lewis Dining Hall");
        assert_eq!(location.id, "40");
    }

    #[test]
    fn test_serde() {
        let x = LocationMeta {
            name: "College Nine/John R. Lewis Dining Hall".into(),
            id: "40".into(),
            url: "https://nutrition.sa.ucsc.edu/shortmenu.aspx?\
            sName=UC+Santa+Cruz+Dining&\
            locationNum=40&\
            locationName=College+Nine/John+R.+Lewis+Dining+Hall&naFlag=1"
                .parse()
                .unwrap(),
        };
        let serialized = serde_json::to_string(&x).unwrap();
        let deserialized: LocationMeta = serde_json::from_str(&serialized).unwrap();
        assert_eq!(x, deserialized);
    }
}
