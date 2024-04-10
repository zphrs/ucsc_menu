use chrono::NaiveDate;

use super::meal::Meal;
use crate::parse::Error;
#[derive(Debug)]
pub struct DailyMenu<'a> {
    date: NaiveDate,
    meals: Vec<Meal<'a>>,
}

impl<'a> DailyMenu<'a> {
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // TODO
        todo!("Implement the MealsOnDate::from_html_element function")
    }
}

#[cfg(test)]
mod tests {
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
        println!("{:?}", meals.meals)
    }
}
