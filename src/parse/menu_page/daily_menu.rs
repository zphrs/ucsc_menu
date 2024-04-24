use chrono::NaiveDate;
use juniper::marker::IsInputType;
use juniper::{graphql_interface, graphql_object};

use super::meal::{Meal, MealType};
use crate::parse::Error;
use crate::static_selector;

use regex::Regex;

#[derive(Debug, Clone)]
pub struct DailyMenu<'a> {
    // graphql representation: yyyy-MM-dd
    date: NaiveDate,
    meals: Vec<Meal<'a>>,
}

#[graphql_object]
impl<'a> DailyMenu<'a> {
    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn meals(&self, meal_type: Option<MealType>) -> Vec<Meal<'a>> {
        match meal_type {
            Some(meal_type) => self
                .meals
                .iter()
                .filter(|meal| meal.meal_type == meal_type)
                .cloned()
                .collect::<Vec<_>>(),
            None => self.meals.clone(),
        }
    }
}

impl PartialEq for DailyMenu<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

impl PartialOrd for DailyMenu<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for DailyMenu<'_> {}

impl Ord for DailyMenu<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

impl<'a> DailyMenu<'a> {
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        static_selector!(DATE_SELECTOR <- "input[name=strCurSearchDays]");
        static_selector!(MEAL_SELECTOR <- r##"table[bordercolor="#CCC"] table[bordercolor="#FFFF00"]"##);
        let date_str = element
            .select(&DATE_SELECTOR)
            .next()
            .ok_or_else(|| Error::html_parse_error("Date field not found"))?
            .attr("value")
            .ok_or_else(|| Error::html_parse_error("No value on date field"))?;

        let date = NaiveDate::parse_from_str(date_str, "%m/%d/%Y")
            .map_err(|_x| Error::html_parse_error("Date is not in valid format."))?;
        let meals = element
            .select(&MEAL_SELECTOR)
            .map(Meal::from_html_element)
            .collect::<Result<_, Error>>()?;

        Ok(Self { date, meals })
    }
}

#[cfg(test)]
mod tests {
    use juniper::{EmptyMutation, EmptySubscription, RootNode};

    use super::*;
    use std::fs;

    #[test]
    fn test_from_html_element() {
        let html =
            fs::read_to_string("./src/parse/html_examples/daily_menu/meals_on_date.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let meals = DailyMenu::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(
            meals.date,
            NaiveDate::parse_from_str("April 9, 2024", "%B %d, %Y").unwrap()
        );
    }

    #[tokio::test]
    async fn test_schema() {
        let html =
            fs::read_to_string("./src/parse/html_examples/daily_menu/meals_on_date.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let meal = DailyMenu::from_html_element(document.root_element())
            .expect("The example html should be valid");
        let schema = RootNode::new(
            meal,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );
        // println!("{}", schema.as_sdl());
        let query = r#"
            query {
                date
                meals {
                    mealType
                    sections {
                        name
                        foodItems {
                            name
                        }
                    }
                }
            }
        "#;
        let binding = juniper::Variables::default();
        let res = juniper::execute(query, None, &schema, &binding, &())
            .await
            .unwrap()
            .0;
        serde_json::to_string_pretty(&res).unwrap();
        // println!("{:#?}", res);
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
    }
}
