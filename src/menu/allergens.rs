use bitflags::bitflags;

pub struct AllergenInfo(AllergenFlags);

impl AllergenInfo {
    // should pass in the allergen image elements
    pub fn from_html_elements(elements: scraper::element_ref::Select) -> Self {
        // iterate over the allergen image elements and continuously oring the allergen flags
        // use reduce to or the allergen flags
        let allergen_flags = elements
            .map(|element| {
                let img_url = element.value().attr("src").unwrap();
                Self::img_url_to_allergen(img_url)
            })
            .reduce(|acc, flags| acc | flags)
            .unwrap_or(AllergenFlags::empty());
        Self(allergen_flags)
    }
    fn img_url_to_allergen(img_url: &str) -> AllergenFlags {
        // verify that the image url starts with "LegendImages/"
        const PREFIX: &str = "LegendImages/";
        if !img_url.starts_with(PREFIX) {
            return AllergenFlags::empty();
        }
        // chop off the "LegendImages/" prefix
        let img_url = &img_url[PREFIX.len()..];
        // verify that the image url ends with ".gif"
        const SUFFIX: &str = ".gif";
        if !img_url.ends_with(SUFFIX) {
            return AllergenFlags::empty();
        }
        // chop off the ".gif" suffix
        let img_url = &img_url[..img_url.len() - SUFFIX.len()];
        match img_url {
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
            _ => AllergenFlags::empty(),
        }
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

bitflags! {
    pub struct AllergenFlags: u16 {
        const Egg = 0b0000000000000001;
        const Fish = 0b0000000000000010;
        const GlutenFriendly = 0b0000000000000100;
        const Milk = 0b000000000001000;
        const Peanut = 0b0000000000010000;
        const Soy = 0b0000000000100000;
        const TreeNut = 0b0000000001000000;
        const Alcohol = 0b0000000010000000;
        const Vegan = 0b0000000100000000;
        const Vegetarian = 0b0000001000000000;
        const Pork = 0b0000010000000000;
        const Beef = 0b0000100000000000;
        const Halal = 0b0001000000000000;
        const Shellfish = 0b0010000000000000;
        const Sesame = 0b0100000000000000;
    }
}

#[cfg(test)]

mod tests {
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
        );
        assert!(allergen_info.0.is_all());
    }

    // tests that all the image urls on the allergen page are properly converted to allergen flags
    #[test]
    fn test_img_url_to_allergen() {
        // source: https://nutrition.sa.ucsc.edu/allergenfilter.aspx?strcurlocationnum=40

        let doc = scraper::Html::parse_document(HTML);
        let selector = scraper::Selector::parse("img").unwrap();
        let mut all_allergen_flags = AllergenFlags::empty();
        for element in doc.select(&selector) {
            let img_url = element.value().attr("src").unwrap();
            let allergen_flags = AllergenInfo::img_url_to_allergen(img_url);
            // ensure that the allergen_flags aren't empty
            println!("img_url: {}", img_url);
            assert!(!allergen_flags.is_empty());
            all_allergen_flags |= allergen_flags;
        }
        // ensure that all the allergen flags are picked up properly
        assert!(all_allergen_flags.is_all())
    }
}
