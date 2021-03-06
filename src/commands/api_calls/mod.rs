pub mod qod_api;

use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Deserialize)]
struct Obj {
    status: String,
    copyright: String,
    num_results: u16,
    results: Vec<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct BookDetailsType {
    list_name: String,
    display_name: String,
    bestsellers_date: String,
    published_date: String,
    rank: u8,
    rank_last_week: u8,
    weeks_on_list: u8,
    asterisk: u8,
    dagger: u8,
    amazon_product_url: String,
    isbns: Vec<HashMap<String, String>>,
    book_details: Vec<HashMap<String, String>>,
    reviews: Vec<HashMap<String, String>>,
}
#[derive(Debug, Deserialize)]
struct ListsType {
    status: String,
    copyright: String,
    num_results: u16,
    last_modified: String,
    results: Vec<BookDetailsType>,
}

impl ListsType {
    fn top_book_details(&self) -> Option<Vec<(String, String, String)>> {
        let mut list_books: Vec<(String, String, String)> = Vec::new();
        for category in &self.results {
            for top_books in &category.book_details {
                list_books.push((
                    top_books.get("title").unwrap().to_string(),
                    top_books.get("author").unwrap().to_string(),
                    top_books.get("description").unwrap().to_string(),
                ));
            }
        }
        if list_books.len() > 0 {
            Some(list_books)
        } else {
            None
        }
    }
}

impl Obj {
    fn get_list_names(&self) -> Option<Vec<String>> {
        let mut list_names: Vec<String> = Vec::new();
        for pairs in &self.results {
            if pairs.contains_key("list_name") {
                list_names.push(pairs.get("list_name").unwrap().clone());
            }
        }
        if list_names.len() > 0 {
            Some(list_names)
        } else {
            None
        }
    }
}

pub async fn get_genre_lists() -> Result<Vec<String>, String> {
    let nyt_token = env::var("NYT_TOKEN").unwrap();
    let list_names = reqwest::get(format!(
        "https://api.nytimes.com/svc/books/v3/lists/names.json?api-key={}",
        &nyt_token
    ))
    .await
    .expect("Cannot reach the API servers")
    .json::<Obj>()
    .await
    .expect("Cannot deserialize")
    .get_list_names()
    .unwrap();
    Ok(list_names)
}

pub async fn get_top_books(list_name: &str) -> Result<Vec<(String, String, String)>, String> {
    let nyt_token = env::var("NYT_TOKEN").unwrap();
    match reqwest::get(format!(
        "https://api.nytimes.com/svc/books/v3/lists.json?list={}&api-key={}",
        &list_name, &nyt_token
    ))
    .await
    {
        Ok(resp) => match resp.json::<ListsType>().await {
            Ok(resp) => match resp.top_book_details() {
                Some(resp) => return Ok(resp),
                None => return Err("No books found in that genre".to_string()),
            },
            Err(_) => return Err("Error Parsing JSON".to_string()),
        },
        Err(_) => return Err("No books found in that genre".to_string()),
    }
}
