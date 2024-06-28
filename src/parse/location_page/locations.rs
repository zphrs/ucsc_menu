use std::slice::{Iter, IterMut};

use chrono::NaiveDate;
use juniper::{graphql_object, GraphQLInputObject};
use scraper::Html;

use crate::parse::menu_page::DailyMenu;
use crate::{parse::Error, static_selector};

use super::location_meta::LocationMeta;

use super::location_data::LocationData;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct Location<'a>(LocationData<'a>, LocationMeta);

#[derive(GraphQLInputObject, Debug)]
pub struct DateRange {
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
}

#[graphql_object]
impl<'a> Location<'a> {
    pub fn id(&self) -> &str {
        self.1.id()
    }
    pub fn name(&self) -> &str {
        self.1.name()
    }
    #[allow(clippy::needless_pass_by_value)] // ignored because graphql doesn't support pass by reference
    pub fn menus(&self, date_range: Option<DateRange>) -> Vec<&DailyMenu<'a>> {
        if let Some(DateRange { start, end }) = date_range {
            self.0
                .menus()
                .filter(|x| {
                    let mut incl = true;
                    incl &= start.map_or(true, |start_date| x.date() >= start_date);
                    incl &= end.map_or(true, |end_date| x.date() <= end_date);
                    incl
                })
                .collect()
        } else {
            self.0.menus().collect()
        }
    }
}

impl<'a> Location<'a> {
    pub fn new(location_meta: LocationMeta) -> Self {
        Self(LocationData::new(), location_meta)
    }

    pub fn add_meals<'b: 'a>(
        &mut self,
        htmls: impl Iterator<Item = &'b Html>,
    ) -> Result<(), Error> {
        // TODO: instead of immediately clearing, diff the similar meals first
        self.clear();
        for html in htmls {
            self.0.add_meal(html)?;
        }
        Ok(())
    }

    pub const fn metadata(&self) -> &LocationMeta {
        &self.1
    }
    #[cfg(test)]
    pub fn hydrated(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, Clone)]
pub struct Locations<'a> {
    locations: Vec<Location<'a>>,
}

#[graphql_object]
impl<'a> Locations<'a> {
    #[allow(clippy::needless_pass_by_value)] // ignored because graphql doesn't support pass by reference
    pub fn locations(&self, ids: Option<Vec<String>>) -> Vec<&Location<'a>> {
        ids.map_or_else(
            || self.locations.iter().collect(),
            |ids| {
                let ids: Vec<&str> = ids.iter().map(std::string::String::as_str).collect();
                self.locations
                    .iter()
                    .filter(|location| ids.contains(&location.1.id()))
                    .collect()
            },
        )
    }
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

    pub fn iter_mut(&mut self) -> IterMut<Location<'a>> {
        self.locations.iter_mut()
    }

    pub fn iter(&self) -> Iter<Location<'a>> {
        self.locations.iter()
    }
    // might eventually be used for diffing
    #[cfg(unused)]
    pub fn add_meals<'b: 'a>(
        &mut self,
        htmls: impl Iterator<Item = &'b Html>,
        location_meta: &LocationMeta,
    ) -> Result<(), Error> {
        let location = self
            .locations
            .iter_mut()
            .find(|x| x.1 == *location_meta)
            .ok_or_else(|| {
                Error::Internal(format!(
                    "Location with id {} is either already hydrated or does not exist. Clear all locations and try again.",
                    location_meta.id()
                ))
            })?;

        location.add_meals(htmls)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use juniper::{EmptyMutation, EmptySubscription, RootNode, Variables};
    use url::Url;

    use super::*;
    use std::{collections::HashMap, fs};

    #[test]
    fn test_from_html_element() {
        let html =
            fs::read_to_string("./src/parse/html_examples/locations/locations.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let locations = Locations::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(locations.locations.len(), 14);
        println!("{:#?}", locations.locations);
    }

    #[tokio::test]
    async fn test_location_schema() {
        let html = Html::parse_document(
            &fs::read_to_string("src/parse/html_examples/daily_menu/dining_hall.html").unwrap(),
        );
        let url: Url = "https://nutrition.sa.ucsc.edu/shortmenu.aspx?\
        sName=UC+Santa+Cruz+Dining&\
        locationNum=40&\
        locationName=College+Nine/John+R.+Lewis+Dining+Hall&naFlag=1"
            .parse()
            .expect("url should be valid");
        let mut location = Location::new(LocationMeta::from_url(url).unwrap());
        let v = [html];
        location.add_meals(v.iter()).unwrap();
        assert!(location.hydrated());
        let root = RootNode::new(
            location,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );
        // println!("{}", root.as_sdl());
        let query = r#"
            {
                name
                # If you change the start or end date then the query will return an empty array
                # you can also set either start or end to null or simply omit them to not filter by that constraint
                menus(dateRange: {start: "2024-04-05", end: "2024-04-05"}) { 
                    date
                    meals {
                        mealType
                        sections {
                            name
                            foodItems {
                                name
                                allergens
                            }
                        }
                    }
                }
            }
        "#;
        let binding: Variables = HashMap::default();
        let res = juniper::execute(query, None, &root, &binding, &())
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }

    #[tokio::test]
    async fn test_locations_schema() {
        let html =
            fs::read_to_string("./src/parse/html_examples/locations/locations.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let locations = Locations::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(locations.locations.len(), 14);
        let root = RootNode::new(
            locations,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );
        // println!("{}", root.as_sdl());
        let query = r"
            {
                locations {
                    id
                    name
                }
            }
        ";
        let binding: Variables = HashMap::default();
        let res = juniper::execute(query, None, &root, &binding, &())
            .await
            .unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }
}
