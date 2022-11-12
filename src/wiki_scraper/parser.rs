use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use ego_tree::NodeRef;
use scraper::{Selector, Html, ElementRef, Node};
use itertools::Itertools;


const EFFECT_TRIGGERS: [&str; 5] = ["Start of Turn", "End turn", "Start of battle", "Sell", "Buy"];

lazy_static! {
    static ref TBL_PET_SELECTOR: Selector = Selector::parse("table.fandom-table, content wds-is-current").unwrap();
    static ref TBL_FOOD_SELECTOR: Selector = Selector::parse("table.fandom-table").unwrap();
    static ref ROW_SELECTOR: Selector = Selector::parse("tr").unwrap();
    static ref CELL_SELECTOR: Selector = Selector::parse("td").unwrap();
}

#[derive(Debug, PartialEq)]
pub enum SearchCateg {
    PET,
    FOOD,
}

#[derive(Deserialize, Debug)]
pub struct SAPWikiSources {
    pub pets_tier_1: String,
    pub pets_tier_2: String,
    pub pets_tier_3: String,
    pub pets_tier_4: String,
    pub pets_tier_5: String,
    pub pets_tier_6: String,
    pub food: String
}

pub fn read_wiki_url<P: AsRef<Path>>(path: P) -> Result<SAPWikiSources, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let urls = serde_json::from_reader(reader)?;

    Ok(urls)
}

pub fn get_page_info(url: &str) -> String {
    let response = reqwest::blocking::get(url).unwrap().text().unwrap();
    response
}

fn collect_text<'a, I>(vals: I) -> Vec<String>
where
    I: Iterator<Item = NodeRef<'a, Node>>,
{
    vals.filter_map(|child| ElementRef::wrap(child))
        .flat_map(|el| el.text())
        .filter_map(|elem| {
            if !elem.is_empty() {
                Some(elem.to_string())
            } else {
                None
            }
        })
        .collect_vec()
}

fn largest_table<'a>(doc: &'a Html, table_type: SearchCateg) -> Option<ElementRef<'a>> {
    let selected_tbl = match table_type {
        SearchCateg::FOOD => doc.select(&TBL_PET_SELECTOR),
        SearchCateg::PET => doc.select(&TBL_FOOD_SELECTOR)
    };

    selected_tbl.max_by_key(|table| {
        table.select(&ROW_SELECTOR).count()
    })
}

pub fn parse_page_info(url: &str, search_categ: SearchCateg) {
    let response = get_page_info(url);
    
    let document = Html::parse_document(&response);

    let main_table = largest_table(&document, search_categ)
        .expect("No tables found in document?");

    for row in main_table.select(&ROW_SELECTOR) {
        let entries = row.select(&CELL_SELECTOR).collect::<Vec<_>>();

        for cell in entries.iter() {
            let child_elems = collect_text(cell.children());
            
            if child_elems.len() == 1 {
                let name_or_trigger = child_elems.first().unwrap();
                if !EFFECT_TRIGGERS.contains(&&name_or_trigger[..]) && name_or_trigger.chars().all(char::is_alphabetic){
                    println!("{}", name_or_trigger)
                }
            } else {
                let sibl_elems = collect_text(cell.next_siblings());
                if sibl_elems.len() >= 1 {
                    println!("> {:?}", sibl_elems)
                }
            }
        }
    }

}