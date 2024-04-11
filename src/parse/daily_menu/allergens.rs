use std::fmt::Display;

use crate::parse::Error;
use bitflags::bitflags;
use juniper::graphql_object;

#[derive(Debug, PartialEq, Eq)]
pub struct AllergenInfo(AllergenFlags);

impl AllergenInfo {
    // should pass in the allergen image elements
    pub fn from_html_elements(elements: scraper::element_ref::Select) -> Result<Self, Error> {
        // iterate over the allergen image elements and continuously oring the allergen flags
        // use reduce to or the allergen flags
        let allergen_flags = elements
            .filter_map(|element| element.value().attr("src"))
            .map(Self::img_url_to_allergen);
        // if there is an error, return the error via a for loop
        let mut acc = AllergenFlags::empty();
        for allergen_flag in allergen_flags {
            acc |= allergen_flag?;
        }
        Ok(Self(acc))
    }
    fn img_url_to_allergen(img_url: &str) -> Result<AllergenFlags, Error> {
        // verify that the image url starts with "LegendImages/"
        const PREFIX: &str = "LegendImages/";
        if !img_url.starts_with(PREFIX) {
            return Err(Error::html_parse_error(
                "Allergen image url does not start with LegendImages/",
            ));
        }
        // chop off the "LegendImages/" prefix
        let img_url = &img_url[PREFIX.len()..];
        // verify that the image url ends with ".gif"
        const SUFFIX: &str = ".gif";
        if !img_url.ends_with(SUFFIX) {
            return Err(Error::html_parse_error(
                "Allergen image url does not end with .gif",
            ));
        }
        // chop off the ".gif" suffix
        let img_url = &img_url[..img_url.len() - SUFFIX.len()];
        let res = match img_url {
            "eggs" => AllergenFlags::Egg,
            "fish" => AllergenFlags::Fish,
            "gluten" => AllergenFlags::GlutenFriendly,
            "milk" => AllergenFlags::Milk,
            "nuts" => AllergenFlags::Peanut,
            "soy" => AllergenFlags::Soy,
            "treenut" => AllergenFlags::TreeNut,
            "alcohol" => AllergenFlags::Alcohol,
            "vegan" => AllergenFlags::Vegan,
            "veggie" => AllergenFlags::Vegetarian,
            "pork" => AllergenFlags::Pork,
            "beef" => AllergenFlags::Beef,
            "halal" => AllergenFlags::Halal,
            "shellfish" => AllergenFlags::Shellfish,
            "sesame" => AllergenFlags::Sesame,
            _ => Err(Error::html_parse_error("Unknown allergen image url"))?,
        };
        Ok(res)
    }
    pub fn is_all(&self) -> bool {
        self.0.is_all()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn contains(&self, flags: AllergenFlags) -> bool {
        self.0.contains(flags)
    }
}

#[graphql_object]
impl AllergenInfo {
    pub fn allergens(&self) -> Vec<&'static str> {
        self.into()
    }
}

impl Display for AllergenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&AllergenInfo> for Vec<&'static str> {
    fn from(val: &AllergenInfo) -> Self {
        (&val.0).into()
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq)]
    pub struct AllergenFlags: u16 {
        const Egg = 1;
        const Fish = 1 << 1;
        const GlutenFriendly = 1 << 2;
        const Milk = 1 << 3;
        const Peanut = 1 << 4;
        const Soy = 1 << 5;
        const TreeNut = 1 << 6;
        const Alcohol = 1 << 7;
        const Vegan =  1 << 8;
        const Vegetarian = 1 << 9;
        const Pork = 1 << 10;
        const Beef = 1 << 11;
        const Halal = 1 << 12;
        const Shellfish = 1 << 13;
        const Sesame = 1 << 14;
    }
}

impl From<&AllergenFlags> for Vec<&'static str> {
    fn from(val: &AllergenFlags) -> Self {
        static ALLERGENS: [(AllergenFlags, &str); 15] = [
            (AllergenFlags::Egg, "Egg"),
            (AllergenFlags::Fish, "Fish"),
            (AllergenFlags::GlutenFriendly, "Gluten Friendly"),
            (AllergenFlags::Milk, "Milk"),
            (AllergenFlags::Peanut, "Peanut"),
            (AllergenFlags::Soy, "Soy"),
            (AllergenFlags::TreeNut, "Tree Nut"),
            (AllergenFlags::Alcohol, "Alcohol"),
            (AllergenFlags::Vegan, "Vegan"),
            (AllergenFlags::Vegetarian, "Vegetarian"),
            (AllergenFlags::Pork, "Pork"),
            (AllergenFlags::Beef, "Beef"),
            (AllergenFlags::Halal, "Halal"),
            (AllergenFlags::Shellfish, "Shellfish"),
            (AllergenFlags::Sesame, "Sesame"),
        ];
        ALLERGENS
            .iter()
            .filter_map(|(allergen_flag, allergen_name)| {
                let flag = AllergenFlags::from_bits(allergen_flag.bits())
                    .expect("AllergenFlags should be valid");
                if val.contains(flag) {
                    Some(*allergen_name)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Display for AllergenFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let allergens: Vec<&str> = self.into();
        write!(f, "{}", allergens.join(", "))
    }
}

#[cfg(test)]

mod tests {
    use crate::static_selector;

    use super::*;
    const HTML: &str = r#"
<tbody><tr>
  <td class="datelegendcontainer" colspan="4"><div class="datelegendheader">Legend</div></td>
</tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/eggs.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Egg</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/vegan.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Vegan</span></td>
</tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/fish.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Fish</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/veggie.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Vegetarian</span></td>
</tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/gluten.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Gluten Friendly</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/pork.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Pork</span></td>
</tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/milk.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Milk</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/beef.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Beef</span></td></tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/nuts.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Peanut</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/halal.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Halal</span></td>
</tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/soy.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Soy</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/shellfish.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Shellfish</span></td>
</tr>
<tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/treenut.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Tree Nut</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/sesame.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Sesame</span></td>
</tr>
    <tr>
<td align="left" valign="middle" width="25" class="datelegendcontainer"><img src="LegendImages/alcohol.gif" alt="" width="25" height="25"></td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">Alcohol</span></td>
<td align="left" valign="middle" width="25" class="datelegendcontainer">&nbsp;</td>
<td align="left" valign="middle" class="datelegendcontainer"><span class="datelegendicons">&nbsp;</span></td>
</tr>  
</tbody>
"#;
    #[test]
    fn test_allergen_info_from_html_elements() {
        let doc = scraper::Html::parse_document(HTML);
        let allergen_info = AllergenInfo::from_html_elements(
            doc.root_element()
                .select(&scraper::Selector::parse("img").unwrap()),
        )
        .expect("The example html should be valid");
        assert!(allergen_info.0.is_all());
    }

    // tests that all the image urls on the allergen page are properly converted to allergen flags
    #[test]
    fn test_img_url_to_allergen() {
        // source: https://nutrition.sa.ucsc.edu/allergenfilter.aspx?strcurlocationnum=40

        let doc = scraper::Html::parse_document(HTML);
        static_selector!(DATE_SELECTOR <- "img");
        let mut all_allergen_flags = AllergenFlags::empty();
        for element in doc.select(&DATE_SELECTOR) {
            let img_url = element.value().attr("src").unwrap(); // all img elements should have a src attribute
            let allergen_flags = AllergenInfo::img_url_to_allergen(img_url)
                .expect("All img urls in this example should be valid");
            // ensure that the allergen_flags aren't empty
            println!("img_url: {}", img_url);
            assert!(!allergen_flags.is_empty());
            all_allergen_flags |= allergen_flags;
        }
        // ensure that all the allergen flags are picked up properly
        assert!(all_allergen_flags.is_all());
    }
}
