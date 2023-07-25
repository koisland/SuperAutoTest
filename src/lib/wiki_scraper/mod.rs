use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;

use crate::{
    wiki_scraper::parse_icons::{extract_all_wiki_images, list_page_images},
    CONFIG,
};

use self::parse_icons::WikiImgData;

pub mod common;
pub mod parse_ailment;
pub mod parse_food;
pub mod parse_hard_mode_toys;
pub mod parse_icons;
pub mod parse_names;
pub mod parse_pet;
pub mod parse_tokens;
pub mod parse_toy;

lazy_static! {
    static ref ICON_SET: HashSet<String> = {
        if CONFIG.database.update_on_startup {
            let food_imgs = list_page_images("Food");
            let pet_imgs = list_page_images("Pets");
            let token_imgs = list_page_images("Tokens");
            let toy_imgs = list_page_images("Toys");
            let hard_mode_imgs = list_page_images("Hard Mode (Toys)");

            food_imgs
                .into_iter()
                .chain(pet_imgs)
                .chain(token_imgs)
                .chain(toy_imgs)
                .chain(hard_mode_imgs)
                .collect()
        } else {
            HashSet::default()
        }
    };
    static ref IMG_URLS: HashMap<String, WikiImgData> = {
        if CONFIG.database.update_on_startup {
            // Replace extensions, ICON, and '_' in names.
            let re_icon = regex::Regex::new("_*Icon_*").unwrap();
            let re_url = regex::Regex::new("/revision/latest\\?cb=.*").unwrap();
            let imgs = extract_all_wiki_images();

            imgs.into_iter()
                // Replace extension so items can match.
                .map(|mut data| {
                    let cleaned_name = re_icon.replace(&data.name, " ");
                    // Strip revision information.
                    let cleaned_url = re_url.replace(&data.url, "");
                    data.url = cleaned_url.to_string();
                    (
                        cleaned_name.replace(".png", "").trim().replace('_', " "),
                        data
                    )
                }).collect()
        } else {
            HashMap::default()
        }
    };
}
