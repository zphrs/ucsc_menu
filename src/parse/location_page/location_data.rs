use scraper::Html;

use crate::parse::error::Result;
use crate::parse::menu_page::DailyMenu;
use crate::parse::Error;

pub const NUM_MEALS: usize = 10;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct LocationData<'a> {
    menus: [Option<DailyMenu<'a>>; NUM_MEALS], // keep track of up to 10 days of meals
}

const ARRAY_REPEAT_VALUE: std::option::Option<DailyMenu<'static>> = None;
impl<'a> LocationData<'a> {
    pub fn new() -> Self {
        Self {
            menus: [ARRAY_REPEAT_VALUE; NUM_MEALS],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.menus.iter().all(|x| x.is_none())
    }

    pub fn clear(&mut self) {
        self.menus.iter_mut().for_each(|x| *x = None);
    }

    pub fn menus_mut(&mut self) -> impl Iterator<Item = &mut DailyMenu<'a>> {
        self.menus
            .iter_mut()
            .filter_map(|x| if let Some(meal) = x { Some(meal) } else { None })
    }

    pub fn menus(&self) -> impl Iterator<Item = &DailyMenu<'a>> {
        self.menus.iter().filter_map(|x| x.as_ref())
    }

    pub fn remove_meals_before(&mut self, date: chrono::NaiveDate) {
        for meal in self.menus.iter_mut() {
            if let Some(m) = meal {
                if m.date() < date {
                    *meal = None;
                }
            }
        }
    }

    pub fn add_meal(&mut self, html: &'a Html) -> Result<()> {
        let menu = DailyMenu::from_html_element(html.root_element())?;

        self.menus
            .iter_mut()
            .find(|x| x.is_none())
            .ok_or_else(|| Error::internal_error("No empty slots for meal"))?
            .replace(menu);

        self.menus.sort();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_location_data() {
        let html = Html::parse_document(
            &fs::read_to_string("src/parse/html_examples/daily_menu/dining_hall.html").unwrap(),
        );
        let mut location_data = LocationData::new();

        assert!(location_data.is_empty());

        location_data.add_meal(&html).unwrap();

        assert!(!location_data.is_empty());
        assert_eq!(location_data.menus_mut().count(), 1);
        assert_eq!(
            location_data.menus_mut().next().unwrap().date(),
            chrono::NaiveDate::from_ymd_opt(2024, 4, 5).unwrap()
        );
        location_data.clear();
        assert!(location_data.is_empty());
    }
}
