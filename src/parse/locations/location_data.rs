use scraper::Html;

use crate::parse::daily_menu::DailyMenu;
use crate::parse::error::Result;

const NUM_MEALS: usize = 10;

struct LocationData<'a> {
    meals: [Option<DailyMenu<'a>>; NUM_MEALS], // keep track of up to 10 days of meals
}

const ARRAY_REPEAT_VALUE: std::option::Option<DailyMenu<'static>> = None;
impl<'a> LocationData<'a> {
    fn new() -> Self {
        Self {
            meals: [ARRAY_REPEAT_VALUE; NUM_MEALS],
        }
    }

    fn add_meal(&mut self, html: &'a Html) -> Result<()> {
        let menu = DailyMenu::from_html_element(html.root_element())?;

        let Some(slot) = self.meals.iter_mut().filter(|x| x.is_none()).next() else {
            // TODO: make a new error variant
            panic!("no slots left in buffer")
        };

        slot.replace(menu);

        Ok(())
    }
}
