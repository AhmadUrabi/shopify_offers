use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead};
use std::path::Path;

use std::io::prelude::*;

use crate::Variant;
use crate::Product;

pub fn write_to_jsonln(products: Vec<Product>, map: HashMap<String,String>) {
    let mut file = OpenOptions::new().write(true).append(true).open("tmp/upload.jsonl");
    if file.is_err() {
        file = File::create("tmp/upload.jsonl");
    }
    let mut file = file.unwrap();
    for product in products {
        if map.get(&product.barcode).is_none() {
            continue;
        }
        file.write(format!("{{\"input\":{{\"id\":\"{}\",\"price\":{:.2},\"compareAtPrice\":{:.2}}}}}\n", map.get(&product.barcode).unwrap().replace("/", "\\/"), product.offer_rsp, product.rsp).as_bytes())
            .expect("Unable to write data");
    }
}

pub fn write_to_jsonln_clear(products: Vec<Product>, map: HashMap<String,String>) {
    let mut file = OpenOptions::new().write(true).append(true).open("tmp/upload.jsonl");
    if file.is_err() {
        file = File::create("tmp/upload.jsonl");
    }
    let mut file = file.unwrap();
    for product in products {
        if map.get(&product.barcode).is_none() {
            continue;
        }
        file.write(format!("{{\"input\":{{\"id\":\"{}\",\"price\":{:.2},\"compareAtPrice\":null}}}}\n", map.get(&product.barcode).unwrap().replace("/", "\\/"), product.rsp).as_bytes())
            .expect("Unable to write data");
    }
}


pub fn read_jsonl_to_map(path: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let mut map: HashMap<String, String> = HashMap::new();
    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(ip) = line {
                let data = serde_json::from_str(&ip);
                if data.is_err() {
                    continue;
                }
                let data: Variant = data.unwrap();

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
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error reading JSONL file"));
    }
    Ok(map)
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}