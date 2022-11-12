#[macro_use]
extern crate lazy_static;

mod wiki_scraper;
use crate::wiki_scraper::parser::{read_wiki_url, parse_page_info, SearchCateg};


pub fn main() {
    
    if let Ok(wiki_urls) = read_wiki_url("config/sources.json") {
        let res = parse_page_info(&wiki_urls.pets_tier_1, SearchCateg::PET);
    }
}