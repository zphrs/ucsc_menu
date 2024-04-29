use futures::future::TryJoinAll;
use reqwest::{Client, Error as RequestError};

use crate::parse::{LocationMeta, Locations};

pub async fn fetch_locations_page(client: &reqwest::Client) -> Result<String, RequestError> {
    static URL: &str = "https://nutrition.sa.ucsc.edu/";
    let response = client.get(URL).send().await?;
    response.text().await
}

pub fn make_client() -> reqwest::Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("error building client")
}

pub async fn fetch_location_page(
    client: &reqwest::Client,
    location_meta: &LocationMeta,
    date: Option<chrono::NaiveDate>,
) -> Result<String, RequestError> {
    static COOKIES: &str = "WebInaCartDates=;  WebInaCartMeals=; WebInaCartQtys=; WebInaCartRecipes=; WebInaCartLocation=";
    let id = location_meta.id();
    let cookies = format!("{COOKIES}{id}");
    let mut url = location_meta.url().to_owned();
    if let Some(date) = date {
        url.query_pairs_mut()
            .append_pair("dtdate", date.format("%m/%d/%Y").to_string().as_str());
    }
    client
        .get(url)
        .header("Cookie", cookies)
        .send()
        .await?
        .text()
        .await
}

pub async fn fetch_menus_on_date(
    client: &reqwest::Client,
    locations: &Locations<'_>,
    date: Option<chrono::NaiveDate>,
) -> Result<Vec<String>, RequestError> {
    futures::future::try_join_all(
        locations
            .iter()
            .map(|x| fetch_location_page(&client, x.metadata(), date)),
    )
    .await
}

pub fn date_iter(start: chrono::NaiveDate, count: i64) -> impl Iterator<Item = chrono::NaiveDate> {
    (0..count).map(move |x| start + chrono::Duration::days(x))
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{parse::Locations, transpose::transposed};

    use super::*;
    use futures::{stream::FuturesUnordered, StreamExt};
    use url::Url;

    #[tokio::test]
    async fn test_fetch_locations_page() {
        let start_time = std::time::Instant::now();
        let client = make_client();
        let page = fetch_locations_page(&client).await.unwrap();
        println!(
            "Time taken to get locations page: {:?}",
            start_time.elapsed()
        );
        let parsed = scraper::Html::parse_document(&page);
        let mut locations: Locations = Locations::from_html_element(parsed.root_element()).unwrap();
        println!(
            "Time taken to parse locations page: {:?}",
            start_time.elapsed()
        );
        let todays_menus = fetch_menus_on_date(&client, &mut locations, None)
            .await
            .unwrap();
        println!(
            "Time taken to get today's menus: {:?}",
            start_time.elapsed()
        );
        let parsed_menus = todays_menus
            .iter()
            .map(|x| scraper::Html::parse_document(x))
            .collect::<Vec<_>>();
        for (location, html) in locations.iter_mut().zip(parsed_menus.iter()) {
            location.add_meals(vec![html].iter().map(|x| *x)).unwrap();
        }
        println!(
            "Time taken to parse today's menus: {:?}",
            start_time.elapsed()
        );
        let start_date = chrono::Utc::now().date_naive() - chrono::Duration::days(1);
        let week_menus: FuturesUnordered<_> = date_iter(start_date, 10)
            .map(|x| fetch_menus_on_date(&client, &locations, Some(x)))
            .collect();
        let week_menus: Vec<_> = week_menus.collect().await;
        println!(
            "Time taken to fetch all menus: {:?}. Number of menus: {}",
            start_time.elapsed(),
            week_menus.len() * locations.iter().len()
        );
        let parsed_week_menus = week_menus
            .iter()
            .map_while(|x| {
                Some(
                    x.as_ref()
                        .ok()?
                        .iter()
                        .map(|y| scraper::Html::parse_document(y))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        println!("Time taken to parse all menus: {:?}", start_time.elapsed());
        let parsed_week_menus = transposed(parsed_week_menus);
        for (location, htmls) in locations.iter_mut().zip(parsed_week_menus.iter()) {
            location.add_meals(htmls.iter()).unwrap();
        }
        println!("Time taken to add all menus: {:?}", start_time.elapsed());
        // println!("{:#?}", locations);
        // save the locations to a file
        let locations = serde_json::to_string(&locations).unwrap();
        std::fs::write("locations.json", &locations).unwrap();
        // test that the locations can be deserialized
        let _locations: Locations = serde_json::from_str(&locations).unwrap();
    }

    #[tokio::test]
    async fn test_fetch_location_page() {
        let client = make_client();
        let url: Url = "https://nutrition.sa.ucsc.edu/shortmenu.aspx?\
        sName=UC+Santa+Cruz+Dining&\
        locationNum=40&\
        locationName=College+Nine/John+R.+Lewis+Dining+Hall&naFlag=1"
            .parse()
            .expect("url should be valid");
        let location_meta = LocationMeta::from_url(url).expect("location meta should be valid");
        let page = fetch_location_page(&client, &location_meta, None)
            .await
            .unwrap();
        println!("{}", page);
    }
}
