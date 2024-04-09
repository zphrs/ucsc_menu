use std::{iter::Peekable, vec};

use scraper::{element_ref::Select, selectable::Selectable, Selector};

use crate::{get_or_init_selector, menu::text_from_selection::text_from_selection};

use super::{error::Error, food_item::FoodItem};

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    LateNight,
    Menu,    // used for menus that are not specific to a meal time. Ex: Global Cafe
    Unknown, // used for when the meal type is not known (ex. when the food item is detached from a meal)
    AllDay,  // default if the above don't match
}
#[derive(Debug)]
pub struct Meal<'a> {
    pub meal_type: MealType,
    pub sections: Vec<MealSection<'a>>,
}

impl<'a> Meal<'a> {
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // example html div element at ./html_examples/meal.html
        static ROW_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let row_selector = get_or_init_selector!(
            ROW_SELECTOR,
            r##"table[bordercolor="#FFFF00"] > tbody > tr"##
        );
        let mut top_level_row_iter = element.select(row_selector);
        let meal_name_row = top_level_row_iter.next().ok_or_else(|| {
            Error::html_parse_error("The meal should have a row for the meal type.")
        })?;
        let meal_item_row = top_level_row_iter.next().ok_or_else(|| {
            Error::html_parse_error("The meal should have a row for the meal items.")
        })?;
        static MEAL_TYPE_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let meal_type_selector = get_or_init_selector!(MEAL_TYPE_SELECTOR, ".shortmenumeals");
        let meal_type =
            text_from_selection(&meal_type_selector, meal_name_row, "meal", "meal type")?;
        // print out meal type
        let meal_type = match meal_type {
            "Breakfast" => MealType::Breakfast,
            "Lunch" => MealType::Lunch,
            "Dinner" => MealType::Dinner,
            "Late Night" => MealType::LateNight,
            "Menu" => MealType::Menu,
            _ => MealType::AllDay,
        };

        static SECTION_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let section_selector = get_or_init_selector!(SECTION_SELECTOR, "table > tbody > tr");
        let section_elements = meal_item_row.select(&section_selector);
        let sections = SectionIterator::new(section_elements.peekable(), meal_type);
        let mut sections_vec: Vec<MealSection> = vec![];
        for section in sections {
            match section {
                Ok(section) => {
                    // add section to meal
                    sections_vec.push(section);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(Meal {
            meal_type,
            sections: sections_vec,
        })
    }
}

pub struct SectionIterator<'a> {
    elements: Peekable<Select<'a, 'a>>,
    meal_type: MealType,
}

impl<'a> SectionIterator<'a> {
    pub fn new(elements: Peekable<Select<'a, 'a>>, meal_type: MealType) -> Self {
        SectionIterator {
            elements,
            meal_type,
        }
    }
}

impl<'a> Iterator for SectionIterator<'a> {
    type Item = Result<MealSection<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut elements = &mut self.elements;
        // check if there are any elements left
        if elements.peek().is_none() {
            return None;
        }
        let section = MealSection::from_html_elements(&mut elements, self.meal_type);
        Some(section)
    }
}

#[derive(Debug)]
pub struct MealSection<'a> {
    pub name: &'a str,
    pub food_items: Vec<FoodItem<'a>>,
}

impl<'a> MealSection<'a> {
    // takes in an iterator of tr elements of a specific meal and consumes the elements to create a MealSection
    pub fn from_html_elements(
        elements: &mut Peekable<Select<'a, 'a>>,
        meal_type: MealType,
    ) -> Result<Self, Error> {
        static MEAL_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let section_name_selector =
            get_or_init_selector!(MEAL_NAME_SELECTOR, ".shortmenucats > span");

        // if the first element does not match the section name selector, then return an error
        let first_element = elements.next().ok_or_else(|| {
            Error::html_parse_error("Every section should have a name as the first element.")
        })?;
        let name = text_from_selection(section_name_selector, first_element, "section", "name")?;

        // trim off first and last three characters since the name looks like -- name --
        let name = &name[3..name.len() - 3];

        // iterate through by peeking and calling handle_element
        let mut food_items = vec![];
        while let Some(element) = elements.peek() {
            if element.select(&section_name_selector).next().is_some() {
                break;
            }
            if let Some(food_item) = Self::handle_element(*element, meal_type, name)? {
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
        static SECTION_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
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
        assert_eq!(meal.sections.len(), 3);
        // print out the names of the sections
        println!("{:#?}", meal);
    }
}
