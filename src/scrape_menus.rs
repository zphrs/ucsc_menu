///! This module scrapes https://dining.ucsc.edu to figure out the menus of each dining hall and cafe location.
///! It uses the `reqwest` and `scraper` crates to scrape the website.
use reqwest::Client;
use scraper::{Html, Selector};
