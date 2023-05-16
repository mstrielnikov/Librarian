use elasticsearch::{
    http::ContentType,
    prelude::*,
};
use pdf_extract::extract_text;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
struct Page {
    page_number: i32,
    text: String,
}

impl Page {
    fn as_doc(&self, index_name: &str, title: &str) -> UpdateParts {
        UpdateParts::UpdateDocument(index_name, title).doc(json!({
            "text": self.text,
            "page_number": self.page_number,
        }))
    }
}

#[derive(Serialize, Deserialize)]
struct PdfFileIndex {
    index_name: String,
    index_type: String,
    language: String,
    pages: Vec<Page>,
    title: String,
    author: String,
    page_num: i32,
}

impl PdfFileIndex {
    fn as_docs(&self) -> Vec<UpdateParts> {
        self.pages
            .iter()
            .map(|p| p.as_doc(&self.index_name, &self.title))
            .collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_name = "";
    let index_type = "";
    let load_path = env::args().nth(1).expect("PDF file path not provided");

    let pdf_file = File::open(load_path)?;
    let pdf_reader = BufReader::new(pdf_file);
    let parsed_pages = extract_text(pdf_reader)?
        .split('\n')
        .enumerate()
        .map(|(index, text)| Page {
            page_number: index as i32 + 1,
            text: text.to_owned(),
        })
        .collect();

    let pdf_info = pdf_extract::info::get_info(pdf_reader)?;
    let title = pdf_info.title.unwrap_or_default();
    let author = pdf_info.author.unwrap_or_default();
    let page_num = pdf_info.pages as i32;

    let pdf_file_index = PdfFileIndex {
        index_name: index_name.to_owned(),
        index_type: index_type.to_owned(),
        language: "".to_owned(),
        pages: parsed_pages,
        title,
        author,
        page_num,
    };

    let mut client = SyncClientBuilder::new()
        .static_node("http://host1:9200")?
        .build()?;

    let bulk_request = BulkParts::new()
        .body(pdf_file_index.as_docs())
        .content_type(ContentType::JSON);

    let response = client.document().bulk(bulk_request)?;

    match response.errors() {
        true => println!("Failure: {:?}", response),
        false => println!("Indexed {} items", response.len()),
    }

    Ok(())
}
