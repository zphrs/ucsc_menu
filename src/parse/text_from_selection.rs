use super::Error;
use scraper::{ElementRef, Selector};

/// will panic if the selector does not match at least one element or if there is not exactly one text node inside the element
pub fn text_from_selection<'a>(
    selector: &Selector,
    element: ElementRef<'a>,
    parent_label: &str,
    child_label: &str,
) -> Result<&'a str, Error> {
    let parent = element
        .select(selector)
        .next() // first match
        .ok_or_else(|| {
            Error::HTMLParseError(format!(
                "Every {parent_label} element should have a {child_label}."
            ))
        })?;
    get_inner_text(parent, child_label)
}

/// will panic if ther is not exactly one text node inside the element
pub fn get_inner_text<'a>(element: ElementRef<'a>, text_label: &str) -> Result<&'a str, Error> {
    let mut text_iter = element.text();
    let text_node = text_iter.next().ok_or_else(|| {
        Error::TextNodeParseError(format!("{text_label} should have text inside."))
    })?;

    if text_iter.next().is_some() {
        // capitalize the first letter of the text node
        let mut text_label = text_label.to_string();
        text_label[..1].make_ascii_uppercase();
        return Err(Error::TextNodeParseError(format!(
            "{text_label} element should only have one text node inside of it."
        )));
    }
    Ok(text_node)
}
