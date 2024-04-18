use super::allergens::{AllergenFlags, AllergenInfo, Allergens};
use crate::parse::text_from_selection::{get_inner_text, text_from_selection};
use crate::parse::Error;
use crate::static_selector;
use juniper::graphql_object;
use rusty_money::{iso, Money};

#[derive(Debug, Clone)]
pub struct FoodItem<'a> {
    name: &'a str,
    allergen_info: AllergenInfo,
    price: Option<Money<'a, iso::Currency>>, // in cents
}

impl PartialEq for FoodItem<'_> {
    fn eq(&self, other: &Self) -> bool {
        // we ignore meal_type, category and price intentionally in checking equality
        self.name == other.name && self.allergen_info == other.allergen_info
    }
}

impl Eq for FoodItem<'_> {}

impl<'a> FoodItem<'a> {
    pub fn from_html_element(element: scraper::ElementRef<'a>) -> Result<Self, Error> {
        // example html tr element at ./html_examples/food_item.html

        // get name with css selector .shortmenurecipes > span
        static_selector!(NAME_SELECTOR <- ".shortmenurecipes > span");
        let name = text_from_selection(&NAME_SELECTOR, element, "foodItem", "name")?.trim_end();
        // get allergen info with css selector td > img
        static_selector!(ALLERGEN_INFO_SELECTOR <- "td > img");
        let allergen_info =
            AllergenInfo::from_html_elements(element.select(&ALLERGEN_INFO_SELECTOR))?;

        // try to get price with css selector .shortmenuprices > span
        static_selector!(PRICE_SELECTOR <- ".shortmenuprices > span");
        let price_element = element.select(&PRICE_SELECTOR).next();
        let price = if let Some(price_element) = price_element {
            let price = get_inner_text(price_element, "price")?; // will look like "$5.00"
                                                                 // if price is equal to &nbsp; then return None
            if price == "\u{00A0}" {
                None
            } else {
                let price = &price[1..]; // remove the dollar sign
                Some(Money::from_str(price, iso::USD)?)
            }
        } else {
            None
        };

        Ok(Self {
            name,
            allergen_info,
            price,
        })
    }

    pub fn get_allergen_mask(&self) -> AllergenFlags {
        self.allergen_info.into()
    }
}

impl From<Vec<Allergens>> for AllergenFlags {
    fn from(allergens: Vec<Allergens>) -> Self {
        allergens
            .into_iter()
            .fold(AllergenFlags::empty(), |acc, x| acc | x.into())
    }
}

#[graphql_object]
impl<'a> FoodItem<'a> {
    pub fn allergens(&self) -> Vec<Allergens> {
        self.allergen_info.into()
    }

    pub fn name(&self) -> &'a str {
        self.name
    }
    pub fn price(&self) -> Option<String> {
        self.price.as_ref().map(Money::to_string)
    }
}

#[cfg(test)]
mod tests {
    use juniper::{EmptyMutation, EmptySubscription, RootNode};

    use super::*;
    use crate::parse::menu_page::allergens::{AllergenFlags, AllergenInfo};

    #[test]
    fn test_food_item_from_html_element() {
        // source: https://nutrition.sa.ucsc.edu/menuSamp.asp?locationNum=40&locationName=Colleges+Nine+%26+Ten&sName=&naFlag=
        // load the html file
        let html =
            std::fs::read_to_string("./src/parse/html_examples/daily_menu/food_item.html").unwrap(); // file system should be reliable
        let doc = scraper::Html::parse_document(&html);
        let food_item = FoodItem::from_html_element(doc.root_element())
            .expect("The example html should be valid");
        assert_eq!(food_item.name(), "Cream Cheese pck");
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

    #[tokio::test]
    async fn test_schema() {
        let x = FoodItem {
            name: "yummy meat",
            allergen_info: AllergenInfo(AllergenFlags::Egg | AllergenFlags::Sesame),
            price: None,
        };
        let rn = RootNode::new(
            x,
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        );

        println!("{}", rn.as_sdl());
        let query = r#"
            {
                allergens
                name
                price
            }
        "#;
        let binding = juniper::Variables::default();
        let result = juniper::execute(query, None, &rn, &binding, &())
            .await
            .unwrap();
        println!("{:?}", result);
        // assert!(false); // just to see the output
        // TODO: delete the above line test
    }
}
