use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead};
use std::path::Path;

use std::io::prelude::*;

use crate::Variant;
use crate::Product;

pub fn write_to_jsonln(products: Vec<Product>, map: HashMap<String,String>) {
    let mut file = OpenOptions::new().write(true).append(true).open("tmp/upload.jsonl").unwrap();
    for product in products {
        if map.get(&product.barcode).is_none() {
            continue;
        }
        file.write(format!("{{\"input\":{{\"id\":\"{}\",\"price\":{:.2},\"compareAtPrice\":{:.2}}}}}\n", map.get(&product.barcode).unwrap().replace("/", "\\/"), product.offer_rsp, product.rsp).as_bytes()).unwrap();
    }
}


pub fn read_jsonl_to_map(path: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    if let Ok(lines) = read_lines(path) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                let data: Variant = serde_json::from_str(&ip).unwrap();
                if data.barcode.is_none() && data.price.is_none() && data.compareAtPrice.is_none() {
                    continue;
                } else {
                    if data.barcode.is_some() {
                        map.insert(data.barcode.unwrap(), data.id);
                    }
                }
            }
        }
    } else {
        println!("Error reading JSONL file");
    }
    map
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}