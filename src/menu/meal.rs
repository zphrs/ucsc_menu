use std::{iter::Peekable, vec};

use scraper::{element_ref::Select, Selector};

use super::food_item::FoodItem;

use std::sync::OnceLock;

#[derive(Clone, Copy)]
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
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Self {
        // example html div element at ./html_examples/meal.html
        static MEAL_TYPE_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let meal_type_selector = MEAL_TYPE_SELECTOR.get_or_init(|| {
            Selector::parse(".shortmenutitles > span").expect("Valid meal type selector")
        });
        // get meal name with css selector .shortmenutitles > span
        let meal_type = element
            .select(&meal_type_selector)
            .next()
            .expect("Every meal div should have a meal type.")
            .text()
            .next()
            .expect("Meal type");

        let meal_type = match meal_type {
            "Breakfast" => MealType::Breakfast,
            "Lunch" => MealType::Lunch,
            "Dinner" => MealType::Dinner,
            "Late Night" => MealType::LateNight,
            "Menu" => MealType::Menu,
            _ => MealType::AllDay,
        };

        Meal {
            meal_type,
            sections: vec![],
        }
    }
}

pub struct MealSection<'a> {
    pub name: &'a str,
    pub food_items: Vec<FoodItem<'a>>,
}

static SECTION_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();

impl<'a> MealSection<'a> {
    // takes in an iterator of tr elements of a specific meal and consumes the elements to create a MealSection
    pub fn from_html_element(elements: &mut Peekable<Select<'a, 'a>>) -> Self {
        let section_name_selector = SECTION_NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".shortmenucats > span").expect("Valid section name selector")
        });

        // if the first element does not match the section name selector, then panic
        let first_element = elements.next().expect("Every section should have a name.");
        let name = first_element
            .select(&section_name_selector)
            .next()
            .expect("Every section should have a name.")
            .text()
            .next()
            .expect("Name");

        // trim off first and last three characters
        let name = &name[3..name.len() - 3];

        // iterate through by peeking and calling handle_element
        let mut food_items = vec![];
        while let Some(element) = elements.peek() {
            if let Some(food_item) = Self::handle_element(*element, MealType::AllDay, name) {
                food_items.push(food_item);
            }
            elements.next();
        }
        MealSection { name, food_items }
    }

    fn handle_element(
        element: scraper::ElementRef<'a>,
        meal_type: MealType,
        category: &'a str,
    ) -> Option<FoodItem<'a>> {
        // check if element matches SECTION_NAME_SELECTOR
        let section_name_selector = SECTION_NAME_SELECTOR.get_or_init(|| {
            Selector::parse(".shortmenucats > span").expect("Valid section name selector")
        });
        if element.select(&section_name_selector).next().is_some() {
            return None;
        } else {
            Some(FoodItem::from_html_element(element, category, meal_type))
        }
    }
}
