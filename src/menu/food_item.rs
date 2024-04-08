use crate::get_or_init_selector;
use crate::menu::text_from_selection::{get_inner_text, text_from_selection};

use super::meal::MealType;
use super::{allergens::AllergenInfo, error::Error};
use rusty_money::{iso, Money};
use scraper::Selector;
use std::sync::OnceLock;

pub struct FoodItem<'a> {
    name: &'a str,
    allergen_info: AllergenInfo,
    meal_type: MealType,
    // too many categories to enumerate - custom category per location and varies from day to day
    category: &'a str,
    price: Option<Money<'a, iso::Currency>>, // in cents
}

impl<'a> FoodItem<'a> {
    pub fn from_html_element(
        element: scraper::ElementRef<'a>,
        category: &'a str,
        meal_type: MealType,
    ) -> Result<Self, Error> {
        // example html tr element at ./html_examples/food_item.html

        // get name with css selector .shortmenurecipes > span
        static NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let name_selector = get_or_init_selector!(NAME_SELECTOR, ".shortmenurecipes > span");
        let name = text_from_selection(name_selector, element, "foodItem", "name")?.trim_end();
        // get allergen info with css selector td > img
        static ALLERGEN_INFO_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let allergen_info_selector = get_or_init_selector!(ALLERGEN_INFO_SELECTOR, "td > img");
        let allergen_info =
            AllergenInfo::from_html_elements(element.select(&allergen_info_selector));

        // try to get price with css selector .shortmenuprices > span
        static PRICE_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let price_selector = get_or_init_selector!(PRICE_SELECTOR, ".shortmenuprices > span");
        let price_element = element.select(&price_selector).next();
        let price = if let Some(price_element) = price_element {
            let price: &str = get_inner_text(price_element, "price")?; // will look like "$5.00"
            let price = &price[1..]; // remove the dollar sign
            Some(Money::from_str(price, iso::USD)?)
        } else {
            None
        };

        Ok(Self {
            name,
            allergen_info,
            meal_type,
            category,
            price,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::allergens::AllergenFlags;

    #[test]
    fn test_food_item_from_html_element() {
        // source: https://nutrition.sa.ucsc.edu/menuSamp.asp?locationNum=40&locationName=Colleges+Nine+%26+Ten&sName=&naFlag=
        // load the html file
        let html = std::fs::read_to_string("./src/html_examples/food_item.html").unwrap();
        let doc = scraper::Html::parse_document(&html);
        let food_item =
            FoodItem::from_html_element(doc.root_element(), "Breakfast", MealType::Breakfast)
                .expect("The example html should be valid");
        assert_eq!(food_item.name, "Cream Cheese pck");
        assert!(food_item.allergen_info.contains(AllergenFlags::Vegetarian));
        assert!(food_item.allergen_info.contains(AllergenFlags::Milk));
        assert!(food_item
            .allergen_info
            .contains(AllergenFlags::GlutenFriendly));

        // make sure price is Some(Money::from_str("1.00", iso::USD).unwrap())
        assert_eq!(
            food_item.price,
            Some(Money::from_str("1.00", iso::USD).unwrap())
        );

        // double check meal type and category
    }
}
