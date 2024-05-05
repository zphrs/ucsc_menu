use std::{borrow::Cow, iter::Peekable, sync::OnceLock, vec};

use juniper::{graphql_object, GraphQLEnum, GraphQLObject};
use regex::{Regex, RegexBuilder};
use scraper::{element_ref::Select, selectable::Selectable};

use crate::{
    parse::{remove_excess_whitespace, text_from_selection::text_from_selection},
    static_selector,
};

use super::{
    allergens::{AllergenFlags, Allergens},
    food_item::FoodItem,
};
use crate::parse::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug, GraphQLEnum, serde::Serialize, serde::Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    LateNight,
    Menu,       // used for menus that are not specific to a meal time. Ex: Global Cafe
    Unknown, // used for when the meal type is not known (ex. when the food item is detached from a meal)
    AllDay,  // default if the above don't match
    BananaJoes, // Late Night @ Banana Joes - only for crown
}
#[derive(Debug, GraphQLObject, Clone, serde::Serialize, serde::Deserialize)]
pub struct Meal<'a> {
    pub meal_type: MealType,
    pub sections: Vec<MealSection<'a>>,
}

impl<'a> Meal<'a> {
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // example html div element at ./html_examples/meal.html
        static_selector!(ROW_SELECTOR <- r##"table[bordercolor="#FFFF00"] > tbody > tr"##);
        let mut top_level_row_iter = element.select(&ROW_SELECTOR);
        let meal_name_row = top_level_row_iter.next().ok_or_else(|| {
            Error::html_parse_error("The meal should have a row for the meal type.")
        })?;
        let meal_item_row = top_level_row_iter.next().ok_or_else(|| {
            Error::html_parse_error("The meal should have a row for the meal items.")
        })?;
        static_selector!(MEAL_TYPE_SELECTOR <- ".shortmenumeals");
        let meal_type =
            text_from_selection(&MEAL_TYPE_SELECTOR, meal_name_row, "meal", "meal type")?;
        // print out meal type
        let meal_type = match meal_type {
            "Breakfast" => MealType::Breakfast,
            "Lunch" => MealType::Lunch,
            "Dinner" => MealType::Dinner,
            "Late Night" => MealType::LateNight,
            "Late Night @ Banana Joe's" => MealType::BananaJoes,
            "Menu" => MealType::Menu,
            "All Day" => MealType::AllDay,
            _ => MealType::Unknown,
        };

        static_selector!(SECTION_NAME_SELECTOR <- "table > tbody > tr");
        let section_elements = meal_item_row.select(&SECTION_NAME_SELECTOR);
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
        let elements = &mut self.elements;
        // check if there are any elements left
        elements.peek()?; // if there are no elements left, return None
        let section = MealSection::from_html_elements(elements);
        Some(section)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MealSection<'a> {
    pub name: Cow<'a, str>,
    pub food_items: Vec<FoodItem<'a>>,
}

#[graphql_object]
impl<'a> MealSection<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn food_items(
        &self,
        contains_all_allergens: Option<Vec<Allergens>>,
        excludes_all_allergens: Option<Vec<Allergens>>,
        contains_any_allergens: Option<Vec<Allergens>>,
        name_contains: Option<String>,
    ) -> Vec<FoodItem<'a>> {
        let contains_all_mask: Option<AllergenFlags> = contains_all_allergens.map(|x| x.into());
        let excludes_all_mask: Option<AllergenFlags> = excludes_all_allergens.map(|x| x.into());
        let contains_any_mask: Option<AllergenFlags> = contains_any_allergens.map(|x| x.into());
        let allergen_filter = |food_item: &&FoodItem<'a>| {
            let mask = food_item.get_allergen_mask();
            let mut out = true;
            out &= contains_all_mask.map_or(true, |contains_all| mask.contains(contains_all));
            out &= contains_any_mask.map_or(true, |contains_any| mask.intersects(contains_any));
            out &= excludes_all_mask.map_or(true, |excludes_all| !mask.intersects(excludes_all));
            out
        };

        let name_contains = name_contains.map(|s| {
            RegexBuilder::new(&regex::escape(&s))
                .case_insensitive(true)
                .build()
                .expect("regex using escaped input should be valid")
        });

        self.food_items
            .iter()
            .filter(allergen_filter)
            .filter(|food_item| {
                name_contains
                    .as_ref()
                    .map_or(true, |pat| pat.is_match(food_item.name()))
            })
            .cloned()
            .collect()
    }
}

impl<'a> MealSection<'a> {
    // takes in an iterator of tr elements of a specific meal and consumes the elements to create a MealSection
    pub fn from_html_elements(elements: &mut Peekable<Select<'a, 'a>>) -> Result<Self, Error> {
        static_selector!(SECTION_NAME_SELECTOR <- ".shortmenucats > span");

        // if the first element does not match the section name selector, then return an error
        let first_element = elements.next().ok_or_else(|| {
            Error::html_parse_error("Every section should have a name as the first element.")
        })?;
        let name = text_from_selection(&SECTION_NAME_SELECTOR, first_element, "section", "name")?;

        // trim off first and last three characters since the name looks like -- name --
        let name = &name[3..name.len() - 3];

        let name = remove_excess_whitespace(name);

        // iterate through by peeking and calling handle_element
        let mut food_items = vec![];
        while let Some(element) = elements.peek() {
            if element.select(&SECTION_NAME_SELECTOR).next().is_some() {
                break;
            }
            if let Some(food_item) = Self::handle_element(*element)? {
                food_items.push(food_item);
            }
            elements.next();
            elements.next();
        }
        Ok(MealSection { name, food_items })
    }

    fn handle_element(element: scraper::ElementRef<'a>) -> Result<Option<FoodItem<'a>>, Error> {
        static_selector!(SECTION_NAME_SELECTOR <- ".shortmenucats > span");
        if element.select(&SECTION_NAME_SELECTOR).next().is_some() {
            Ok(None)
        } else {
            let out = FoodItem::from_html_element(element)?;
            Ok(Some(out))
        }
    }
}

#[cfg(test)]
mod tests {
    use juniper::{EmptyMutation, EmptySubscription, RootNode};
    use serde_json::json;

    use super::*;
    use std::{
        fmt::{Debug, Display},
        fs,
    };

    #[test]
    fn test_meal_parse() {
        // load html from "./html_examples/meal.html"
        let html = fs::read_to_string("./src/parse/html_examples/daily_menu/meal.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let meal = Meal::from_html_element(document.root_element())
            .expect("The example html should be valid");
        assert_eq!(meal.meal_type, MealType::Breakfast);
        assert_eq!(meal.sections.len(), 3);
        // print out the names of the sections
        println!("{:#?}", meal.sections);
    }

    #[tokio::test]
    async fn test_graphql_allergen_filtering() {
        let html = fs::read_to_string("./src/parse/html_examples/daily_menu/meal.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let meal = Meal::from_html_element(document.root_element())
            .expect("The example html should be valid");
        let schema = RootNode::new(
            meal,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );
        // println!("{}", schema.as_sdl());
        let query = r"
            {
                mealType
                sections {
                    name
                    foodItems(containsAnyAllergens: [VEGAN, VEGETARIAN], excludesAllAllergens: [MILK]) {
                        name,
                        allergens
                    }
                }
            }
        ";
        let binding = juniper::Variables::default();
        let res = juniper::execute(query, None, &schema, &binding, &())
            .await
            .unwrap()
            .0;
        // println!("{:#?}", res);
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
        // panic!();
    }

    #[tokio::test]
    async fn test_graphql_name_filtering() {
        let html = fs::read_to_string("./src/parse/html_examples/daily_menu/meal.html").unwrap();
        let document = scraper::Html::parse_document(&html);
        let meal = Meal::from_html_element(document.root_element())
            .expect("The example html should be valid");
        let schema = RootNode::new(
            meal,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );
        // println!("{}", schema.as_sdl());
        let query = r#"
            {
                mealType
                sections {
                    name
                    foodItems(nameContains: "muffin") {
                        name
                    }
                }
            }
        "#;
        let binding = juniper::Variables::default();
        let res = juniper::execute(query, None, &schema, &binding, &())
            .await
            .unwrap()
            .0;
        assert_eq!(
            serde_json::to_value(res).expect("json should be valid"),
            json!(
                {
                  "mealType": "BREAKFAST",
                  "sections": [
                    {
                      "name": "Breakfast",
                      "foodItems": []
                    },
                    {
                      "name": "Clean Plate",
                      "foodItems": []
                    },
                    {
                      "name": "Bakery",
                      "foodItems": [
                        {
                          "name": "Blueberry Muffin"
                        },
                        {
                          "name": "Pumpkin Muffin"
                        }
                      ]
                    }
                  ]
            })
        )
    }
}
