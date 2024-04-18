use chrono::NaiveDate;

use super::meal::Meal;
use crate::parse::Error;
use crate::static_selector;

#[derive(Debug)]
pub struct DailyMenu<'a> {
    date: NaiveDate,
    meals: Vec<Meal<'a>>,
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
        static_selector!(MEAL_SELECTOR <- r##"table[bordercolors="#FFFF00"]"##);
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

    pub fn date(&self) -> NaiveDate {
        self.date
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
