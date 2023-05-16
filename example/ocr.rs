use bytes::Bytes;
use rusoto_core::Region;
use rusoto_textract::{AnalyzeDocumentRequest, Block, BlockType, FeatureType, Textract, TextractClient};

use std::env;
use std::fs::read;
use std::path::PathBuf;

fn extract_text_from_pdf(pdf_path: &PathBuf) -> Vec<String> {
    let client = TextractClient::new(Region::default());

    let file_bytes = read(pdf_path).unwrap();
    let document = Bytes::from(file_bytes);

    let feature_types = vec![FeatureType::Forms, FeatureType::Tables];
    let request = AnalyzeDocumentRequest {
        document: Some(document),
        feature_types: Some(feature_types),
        ..Default::default()
    };

    let result = client.analyze_document(request).sync().unwrap();

    result.blocks.unwrap_or_else(Vec::new)
        .into_iter()
        .filter(|block| block.block_type == Some(BlockType::Line))
        .map(|block| block.text.unwrap_or_default())
        .collect()
}

fn main() {
    let pdf_path = env::args().nth(1).expect("PDF file path not provided");
    let text = extract_text_from_pdf(&pdf_path);
    println!("{:?}", text);
}
