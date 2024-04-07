use super::allergens::AllergenInfo;
use super::meal::MealType;
use rusty_money::{iso, Money};
use scraper::Selector;
use std::{borrow::Borrow, cell::OnceCell, sync::OnceLock};

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
    ) -> Self {
        // example html tr element at ./html_examples/food_item.html

        // get name with css selector .shortmenurecipes > span
        static NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let name_selector = NAME_SELECTOR.get_or_init(|| {
            {
                Selector::parse(".shortmenurecipes > span").expect("Valid name selector")
            }
        });
        let name = element
            .select(&name_selector)
            .next()
            .expect("Every fooditem div should have a name.")
            .text()
            .next()
            .expect("Name")
            .trim_end();
        // get allergen info with css selector td > img
        static ALLERGEN_INFO_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let allergen_info_selector = ALLERGEN_INFO_SELECTOR.get_or_init(|| {
            {
                Selector::parse("td > img").expect("Valid allergen info selector")
            }
        });
        let allergen_info =
            AllergenInfo::from_html_elements(element.select(&allergen_info_selector));

        // try to get price with css selector .shortmenuprices > span
        static PRICE_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let price_selector = PRICE_SELECTOR.get_or_init(|| {
            {
                Selector::parse(".shortmenuprices > span").expect("Valid price selector")
            }
        });
        let price_element = element.select(&price_selector).next();
        let price = if let Some(price_element) = price_element {
            let price: &str = price_element.text().next().expect("Price").as_ref(); // will look like "$5.00"
            let price = &price[1..]; // remove the dollar sign
            Some(Money::from_str(price, iso::USD).expect("Price should be a valid number."))
        } else {
            None
        };

        Self {
            name,
            allergen_info,
            meal_type,
            category,
            price,
        }
    }
}

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
            FoodItem::from_html_element(doc.root_element(), "Breakfast", MealType::Breakfast);
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
