#![allow(non_snake_case)]
mod bulk_operations;
mod jsonl;

use calamine::{open_workbook, Error, RangeDeserializerBuilder, Reader, Xlsx};

use dotenv::dotenv;

use std::fs::File;
use std::io::prelude::*;

use rfd::FileDialog;

use serde::{Deserialize, Serialize};

use crate::bulk_operations::{bulk_fetch_operation, bulk_update_operation, upload_file_to_shopify};
use crate::jsonl::{read_jsonl_to_map, write_to_jsonln};

pub struct Product {
    pub barcode: String,
    pub rsp: f64,
    pub offer_rsp: f64,
}


#[derive(Serialize, Deserialize)]
pub struct Variant {
    id: String,
    barcode: Option<String>,
    price: Option<String>,
    compareAtPrice: Option<String>,
    __parentId: Option<String>,
}

fn read_excel_file_and_map(path: String) -> Result<Vec<Product>, Error> {
    let mut res: Vec<Product> = Vec::new();
    let mut workbook: Xlsx<_> = open_workbook(path).unwrap();
    let range = workbook.worksheet_range_at(0);

    if range.is_none() {
        return Err("Error reading excel file".into());
    } 
    let range = range.unwrap();
    if range.is_err() {
        return Err("Error reading excel file".into());
    }
    let range = range.unwrap();
    
    let mut iter = RangeDeserializerBuilder::new().from_range(&range).unwrap();
    loop {
        if let Some(result) = iter.next() {
            let (barcode, _desc, rsp, offer_rsp): (String, String, f64, f64) = result.unwrap();
            res.push(Product {
                barcode: barcode.clone(),
                rsp: rsp,
                offer_rsp: offer_rsp,
            });
        } else {
            break;
        }
    }
    Ok(res)

}

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("Checking Environment Variables");
    if std::env::var("SHOPIFY_API_URL").is_err() {
        println!("SHOPIFY_API_URL not set");
        return;
    }
    if std::env::var("SHOPIFY_API_KEY").is_err() {
        println!("SHOPIFY_API_KEY not set");
        return;
    }
    if std::env::var("SHOPIFY_API_PASSWORD").is_err() {
        println!("SHOPIFY_API_PASSWORD not set");
        return;
    }
    println!("Environment Variables OK");

    println!("Starting");
    
    let upload_key: String;
    let download_url = bulk_fetch_operation().await;

    match download_url {
        Ok(url) => {
            println!("Downloading file");
            let response = reqwest::get(url)
                .await
                .unwrap();

            let mut file = File::create("tmp/res.jsonl").unwrap();
            let body = response.text().await.unwrap();

            file.write_all(body.as_bytes()).unwrap();            
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
    
    let barcode_id_map = read_jsonl_to_map("tmp/res.jsonl");
    if barcode_id_map.is_err() {
        println!("Error reading JSONL file");
        return;
    }
    let barcode_id_map = barcode_id_map.unwrap();
    
    File::create("tmp/upload.jsonl").expect("Unable to create file");
    
    let files = FileDialog::new()
        .add_filter("Excel Files", &["xlsx", "xls"])
        .set_directory("/")
        .pick_files();

    for file in files.unwrap() {
        let file_string = file.into_os_string().into_string();
        if file_string.is_err() {
            println!("Error reading file");
            return;
        }
        let file = file_string.unwrap();

        let products = read_excel_file_and_map(file);
        if products.is_err() {
            println!("Error reading excel file");
            return;
        }
        
        if products.is_err() {
            println!("Error reading excel file");
            return;
        }

        write_to_jsonln(products.unwrap(), barcode_id_map.clone());
    }

    match upload_file_to_shopify().await {
        Ok(key) => {
            upload_key = key;
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    bulk_update_operation(upload_key).await.expect("Error updating products");

    println!("Done");
    println!("Press enter to exit");
    std::io::stdin().read_line(&mut String::new())
      .ok()
      .expect("Failed to read line");
}

