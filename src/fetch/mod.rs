use reqwest::{Client, Error as RequestError};

use crate::parse::LocationMeta;

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
    location_meta: LocationMeta,
) -> Result<String, RequestError> {
    static COOKIES: &str = "WebInaCartDates=;  WebInaCartMeals=; WebInaCartQtys=; WebInaCartRecipes=; WebInaCartLocation=";
    let id = location_meta.id();
    let cookies = format!("{COOKIES}{id}");
    let url = location_meta.url();
    client
        .get(url.clone())
        .header("Cookie", cookies)
        .send()
        .await?
        .text()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[tokio::test]
    async fn test_fetch_locations_page() {
        let client = make_client();
        let _page = fetch_locations_page(&client).await.unwrap();
    }

    async fn test_fetch_location_page() {
        let client = make_client();
        let url: Url = "https://nutrition.sa.ucsc.edu/shortmenu.aspx?\
        sName=UC+Santa+Cruz+Dining&\
        locationNum=40&\
        locationName=College+Nine/John+R.+Lewis+Dining+Hall&naFlag=1"
            .parse()
            .expect("url should be valid");
        let location_meta = LocationMeta::from_url(url).expect("location meta should be valid");
        fetch_location_page(&client, location_meta).await.unwrap();
    }
}
