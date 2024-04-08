use std::{iter::Peekable, vec};

use scraper::{element_ref::Select, Selector};

use crate::{get_or_init_selector, menu::text_from_selection::text_from_selection};

use super::{error::Error, food_item::FoodItem};

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    LateNight,
    Menu,   // used for menus that are not specific to a meal time. Ex: Global Cafe
    AllDay, // default if the above don't match
}

pub struct Meal<'a> {
    pub meal_type: MealType,
    pub sections: Vec<MealSection<'a>>,
}

impl<'a> Meal<'a> {
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // example html div element at ./html_examples/meal.html
        static MEAL_TYPE_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let meal_type_selector = get_or_init_selector!(MEAL_TYPE_SELECTOR, ".shortmenucats > span");
        let meal_type = text_from_selection(&meal_type_selector, element, "meal", "meal type")?;
        // trim off first and last three characters
        let meal_type = &meal_type[3..meal_type.len() - 3];
        let meal_type = match meal_type {
            "Breakfast" => MealType::Breakfast,
            "Lunch" => MealType::Lunch,
            "Dinner" => MealType::Dinner,
            "Late Night" => MealType::LateNight,
            "Menu" => MealType::Menu,
            _ => MealType::AllDay,
        };

        Ok(Meal {
            meal_type,
            sections: vec![],
        })
    }
}

pub struct MealSection<'a> {
    pub name: &'a str,
    pub food_items: Vec<FoodItem<'a>>,
}

static SECTION_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
impl<'a> MealSection<'a> {
    // takes in an iterator of tr elements of a specific meal and consumes the elements to create a MealSection
    pub fn from_html_element(elements: &mut Peekable<Select<'a, 'a>>) -> Result<Self, Error> {
        let section_name_selector =
            get_or_init_selector!(SECTION_NAME_SELECTOR, ".shortmenucats > span");

        // if the first element does not match the section name selector, then return an error
        let first_element = elements.next().ok_or_else(|| {
            Error::html_parse_error("Every section should have a name as the first element.")
        })?;
        let name = text_from_selection(section_name_selector, first_element, "section", "name")?;

        // trim off first and last three characters
        let name = &name[3..name.len() - 3];

        // iterate through by peeking and calling handle_element
        let mut food_items = vec![];
        while let Some(element) = elements.peek() {
            if let Some(food_item) = Self::handle_element(*element, MealType::AllDay, name)? {
                food_items.push(food_item);
            }
            elements.next();
        }
        Ok(MealSection { name, food_items })
    }

    fn handle_element(
        element: scraper::ElementRef<'a>,
        meal_type: MealType,
        category: &'a str,
    ) -> Result<Option<FoodItem<'a>>, Error> {
        // check if element matches SECTION_NAME_SELECTOR
        let section_name_selector =
            get_or_init_selector!(SECTION_NAME_SELECTOR, ".shortmenucats > span");
        if element.select(&section_name_selector).next().is_some() {
            return Ok(None);
        } else {
            let out = FoodItem::from_html_element(element, category, meal_type)?;
            Ok(Some(out))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_meal_parse() {
        // load html from "./html_examples/meal.html"
        let html = fs::read_to_string("./src/html_examples/meal.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let meal = Meal::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(meal.meal_type, MealType::Breakfast);
    }
}
