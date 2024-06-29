use std::{num::NonZeroU32, sync::OnceLock, time::Duration};

use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::InMemoryState,
};
use reqwest::{Client, Error as RequestError};
use tracing::{instrument, Level};

use crate::parse::{LocationMeta, Locations};

pub async fn locations_page(client: &reqwest::Client) -> Result<String, RequestError> {
    static URL: &str = "https://nutrition.sa.ucsc.edu/";
    let response = client.get(URL).send().await?;
    response.text().await
}

pub fn make_client() -> reqwest::Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .gzip(true)
        .build()
        .expect("client creation should succeed")
}

static RATE_LIMIT: u32 = 20;
static DELAY_JITTER: u64 = 2;
static RATE_LIMITER: OnceLock<
    governor::RateLimiter<
        governor::state::NotKeyed,
        InMemoryState,
        QuantaClock,
        NoOpMiddleware<QuantaInstant>,
    >,
> = OnceLock::new();
#[instrument(skip(client, location_meta, date), fields(
    // `%` serializes the peer IP addr with `Display`
    id = %location_meta.id(),
    date = %date.ok_or_else(|| "No date provided").unwrap_or_default().format("%m/%d/%Y"),
), level = Level::TRACE)]
pub async fn fetch_location_page(
    client: &reqwest::Client,
    location_meta: &LocationMeta,
    date: Option<chrono::NaiveDate>,
) -> Result<String, RequestError> {
    let rate_limiter = RATE_LIMITER.get_or_init(|| {
        governor::RateLimiter::direct(governor::Quota::per_second(
            NonZeroU32::new(RATE_LIMIT).unwrap(),
        ))
    });
    let retry_jitter = governor::Jitter::new(Duration::ZERO, Duration::from_secs(DELAY_JITTER));
    rate_limiter.until_ready_with_jitter(retry_jitter).await;
    let id = location_meta.id();
    let cookies = format!("WebInaCartDates=;  WebInaCartMeals=; WebInaCartQtys=; WebInaCartRecipes=; WebInaCartLocation={id}");
    let mut url = location_meta.url().to_owned();
    if let Some(date) = date {
        url.query_pairs_mut()
            .append_pair("dtdate", date.format("%m/%d/%Y").to_string().as_str());
    }
    // println!("Fetching location page for\t{}", location_meta.name());
    let res = client.get(url).header("Cookie", cookies).send().await?;
    let start = std::time::Instant::now();
    let text = res.text().await?;
    log::trace!("Got text of location page in \t {:?}", start.elapsed());
    // gzip decode
    // let html = scraper::Html::parse_document(&text);
    Ok(text)
}

pub async fn menus_on_date(
    client: &reqwest::Client,
    locations: &Locations,
    date: Option<chrono::NaiveDate>,
) -> Result<Vec<String>, RequestError> {
    futures::future::try_join_all(
        locations
            .iter()
            .map(|x| fetch_location_page(client, x.metadata(), date)),
    )
    .await
}

pub fn date_iter(start: chrono::NaiveDate, count: i64) -> impl Iterator<Item = chrono::NaiveDate> {
    (0..count).map(move |x| start + chrono::Duration::days(x))
}

#[cfg(test)]
mod tests {

    use crate::parse::Locations;

    use super::*;

    use url::Url;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fetch_locations_page() {
        // setup_tracing();
        let start_time = std::time::Instant::now();
        let client = make_client();
        let page = locations_page(&client).await.unwrap();
        println!(
            "Time taken to get locations page: {:?}",
            start_time.elapsed()
        );
        let parsed = scraper::Html::parse_document(&page);
        let _locations: Locations = Locations::from_html_element(parsed.root_element()).unwrap();
        println!("Time taken to parse locations:\t{:?}", start_time.elapsed());
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
        println!("{page:#?}");
    }
}
