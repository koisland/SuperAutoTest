use serde::Deserialize;
use serde_json::Value;
use ureq;

use super::ICON_SET;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct WikiImgData {
    pub name: String,
    pub timestamp: String,
    pub url: String,
    pub descriptionurl: String,
    pub descriptionshorturl: String,
    pub ns: usize,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContinueParams {
    pub aicontinue: String,
    pub r#continue: String,
}

const API_ENDPT: &str = "https://superautopets.fandom.com/api.php";
const QRY_ALL_IMGS_PARAMS: [(&str, &str); 4] = [
    ("action", "query"),
    ("format", "json"),
    ("list", "allimages"),
    ("ailimit", "max"),
];

const PRS_PAGE_IMGS_PARAMS: [(&str, &str); 3] =
    [("action", "parse"), ("format", "json"), ("prop", "images")];

/// https://superautopets.fandom.com/api.php?action=parse&format=json&page=Food&prop=images
pub(crate) fn list_page_images(page: &str) -> Vec<String> {
    let req = ureq::get(API_ENDPT)
        .query_pairs(PRS_PAGE_IMGS_PARAMS)
        .query("page", page);

    let resp_str = req.call().unwrap().into_string().unwrap();
    let resp_content: Value = serde_json::from_str(&resp_str).unwrap();

    resp_content
        .get("parse")
        .and_then(|resp| resp.get("images"))
        .and_then(|images| serde_json::from_value::<Vec<String>>(images.clone()).ok())
        .unwrap_or_default()
}

/// Extract all MediaWiki images from SAP wiki.
/// * https://www.mediawiki.org/wiki/API:Main_page
pub(crate) fn extract_all_wiki_images() -> Vec<WikiImgData> {
    let mut all_images: Vec<WikiImgData> = vec![];

    let mut num_pages = 0;
    let mut more_res = None;

    loop {
        let mut req = ureq::get(API_ENDPT).query_pairs(QRY_ALL_IMGS_PARAMS);

        // Ignore check for next page on starting first page until response finished.
        if num_pages != 0 {
            if let Some(Ok(next_page)) = more_res.map(serde_json::from_value::<ContinueParams>) {
                req = req.query("aicontinue", &next_page.aicontinue);
                req = req.query("continue", &next_page.r#continue);
            } else {
                break;
            }
        }

        // Convert response to string.
        let resp_str = req.call().unwrap().into_string().unwrap();
        let resp_content: Value = serde_json::from_str(&resp_str).unwrap();
        // Get image metadat and convert to WikiImgData. Add to growing list.
        if let Some(query_items) = resp_content
            .get("query")
            .and_then(|all_img| all_img.get("allimages"))
        {
            let images: Vec<WikiImgData> = serde_json::from_value(query_items.clone()).unwrap();
            // Only include icon if in icon set for Food, Pets, or Tokens page.
            all_images.extend(
                images
                    .into_iter()
                    .filter(|img| ICON_SET.contains(&img.name)),
            )
        }

        // Set next continue.
        more_res = resp_content.get("continue").cloned();
        num_pages += 1;
    }

    all_images
}
