#![allow(non_snake_case)]
use reqwest::multipart;

// use tokio::time::{sleep, Duration};

use std::fs::File;
use std::io::{self};

use std::io::prelude::*;

pub async fn bulk_fetch_operation() -> Result<String, io::Error>{
    println!("Starting bulk fetch product data operation");

    let result_url: String;
    let graphql_query = format!(
        r#"mutation {{
            bulkOperationRunQuery(
             query: """
              {{
                products {{
                  edges {{
                    node {{
                          id,
                      variants(first:13) {{
                              edges {{
                                  node {{
                                      id,
                                      barcode,
                                      price,
                                      compareAtPrice
                                  }}
                              }}
                          }}
                    }}
                  }}
                }}
              }}
              """
            ) {{
              bulkOperation {{
                id
                status
              }}
              userErrors {{
                field
                message
              }}
            }}
          }}
     "#
    );
    let client = reqwest::Client::new();
    let res = client
        .post(std::env::var("SHOPIFY_API_URL").unwrap())
        .basic_auth(
            std::env::var("SHOPIFY_API_KEY").unwrap(),
            Some(std::env::var("SHOPIFY_API_PASSWORD").unwrap()),
        )
        .body(graphql_query)
        .header("Content-Type", "application/graphql")
        .send()
        .await;
    
    if res.is_err() {
        println!("Reqwest error fetching bulk id");
        return Err(io::Error::new(io::ErrorKind::Other, "Reqwest error fetching bulk id"));
    }
    let res = res.unwrap();

    let body = serde_json::from_str(res.text().await.unwrap().as_ref());
    if body.is_err() {
        println!("Failed to parse bulk id response");
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to parse bulk id response"));
    }
    let body: serde_json::Value = body.unwrap();

    let bulk_id = body["data"]["bulkOperationRunQuery"]["bulkOperation"]["id"].as_str();
    if bulk_id.is_none() {
        println!("Bulk id is none");
        return Err(io::Error::new(io::ErrorKind::Other, "Bulk id is none"));
    } else {
        println!("Bulk id fetched");
        let fetch_body = format!(
            r#"query {{
                node(id: "{}") {{
                  ... on BulkOperation {{
                    url
                    partialDataUrl
                  }}
                }}
              }}
            "#,
            bulk_id.unwrap()
        );

        println!("Waiting for download bulk operation to complete");

        loop {
            let res2 = client
                .post(std::env::var("SHOPIFY_API_URL").unwrap())
                .basic_auth(
                    std::env::var("SHOPIFY_API_KEY").unwrap(),
                    Some(std::env::var("SHOPIFY_API_PASSWORD").unwrap()),
                )
                .body(fetch_body.clone())
                .header("Content-Type", "application/graphql")
                .send()
                .await;
            
            if res2.is_err() {
                println!("Reqwest error fetching download url");
                return Err(io::Error::new(io::ErrorKind::Other, "Reqwest error fetching download url"));
            }
            let res2 = res2.unwrap();

            let body2 =
                serde_json::from_str(res2.text().await.unwrap().as_ref());
            if body2.is_err() {
                println!("Failed to parse download url response");
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to parse download url response"));
            }
            let body2: serde_json::Value = body2.unwrap();
            
            if body2["data"]["node"]["url"].is_null() {
                continue;
            } else {
                result_url = body2["data"]["node"]["url"].as_str().unwrap().to_string();
                println!("Bulk operation completed");
                break;
            }
        }
    }
    Ok(result_url)
}

pub async fn bulk_update_operation(upload_key: String) -> Result<bool, io::Error> {
    println!("Starting bulk update operation");

    let client = reqwest::Client::new();
    let graphql_query = format!(
        r#"mutation {{
            bulkOperationRunMutation(
              mutation: "mutation productVariantUpdate($input: ProductVariantInput!) {{ productVariantUpdate(input: $input) {{ productVariant {{ id,price,compareAtPrice}} userErrors {{field,message}}}}}}",
              stagedUploadPath: "{}"
            ) {{
              bulkOperation {{
                id
                errorCode
                status
              }}
              userErrors {{
                field
                message
              }}
            }}
          }}
     "#,
        upload_key
    );
    let res = client
        .post(std::env::var("SHOPIFY_API_URL").unwrap())
        .basic_auth(
            std::env::var("SHOPIFY_API_KEY").unwrap(),
            Some(std::env::var("SHOPIFY_API_PASSWORD").unwrap()),
        )
        .body(graphql_query)
        .header("Content-Type", "application/graphql")
        .send()
        .await;

    if res.is_err() {
        println!("Reqwest error creating bulk update operation");
        return Err(io::Error::new(io::ErrorKind::Other, "Reqwest error creating bulk update operation"));
    }
    let res = res.unwrap();
      
    
    println!("Update operation started");

    let body = serde_json::from_str(res.text().await.unwrap().as_ref());
    if body.is_err() {
        println!("Failed to parse bulk update response");
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to parse bulk update response"));
    }
    let body: serde_json::Value = body.unwrap();
    let status = body["data"]["bulkOperationRunMutation"]["bulkOperation"]["status"].as_str();
    if status.is_none() {
        println!("Bulk update status is none");
        return Err(io::Error::new(io::ErrorKind::Other, "Bulk update status is none"));
    }

    if body["data"]["bulkOperationRunMutation"]["bulkOperation"]["status"].as_str().unwrap() == "CREATED" {
        println!("Update operation completed");
    } else {
        println!("Update operation failed");
        return Err(io::Error::new(io::ErrorKind::Other, "Update operation failed"));
    }
    Ok(true)
}


pub async fn upload_file_to_shopify() -> Result<String, io::Error> {
    println!("Starting upload file to shopify operation");
    let graphql_query = format!(
        r#"mutation {{
            stagedUploadsCreate(input:{{
                       resource: BULK_MUTATION_VARIABLES,
                       filename: "upload.jsonl",
                       mimeType: "text/jsonl",
                       httpMethod: POST
                     }}){{
                       userErrors{{
                         field,
                         message
                       }},
                       stagedTargets{{
                         url,
                         resourceUrl,
                         parameters {{
                           name,
                           value
                         }}
                       }}
                     }}
                   }}
     "#
    );

    let client = reqwest::Client::new();
    let res = client
        .post(std::env::var("SHOPIFY_API_URL").unwrap())
        .basic_auth(
            std::env::var("SHOPIFY_API_KEY").unwrap(),
            Some(std::env::var("SHOPIFY_API_PASSWORD").unwrap()),
        )
        .body(graphql_query)
        .header("Content-Type", "application/graphql")
        .send()
        .await;

    if res.is_err() {
        println!("Reqwest error creating upload");
        return Err(io::Error::new(io::ErrorKind::Other, "Reqwest error creating upload"));
    }
    let res = res.unwrap();
        
    let body = res.text().await;
    if body.is_ok() {
        let body: serde_json::Value = serde_json::from_str(body.unwrap().as_ref()).unwrap();
        let url = body["data"]["stagedUploadsCreate"]["stagedTargets"][0]["url"].as_str();
        if url.is_none() {
            println!("Upload url is none");
            return Err(io::Error::new(io::ErrorKind::Other, "Upload url is none"));
        }
        let url = url.unwrap();
        
        let parameters = body["data"]["stagedUploadsCreate"]["stagedTargets"][0]["parameters"].as_array();
        if parameters.is_none() {
            println!("Parameters is none");
            return Err(io::Error::new(io::ErrorKind::Other, "Parameters is none"));
        }
        let parameters = parameters.unwrap();

        let file = File::open("tmp/upload.jsonl");
        if file.is_err() {
            println!("Failed to open file");
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to open file"));
        }
        let file = file.unwrap();

        // read file body stream
        
        let some_file = multipart::Part::bytes(file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>() )
        .file_name("tmp/upload.jsonl")
        .mime_str("text/jsonl");

      if some_file.is_err() {
        println!("Failed to parse file for multipart");
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to parse file for multipart"));
      }
      let some_file = some_file.unwrap();

        let mut form = reqwest::multipart::Form::new();
        form = form.text("key", parameters[3]["value"].as_str().unwrap().to_string());
        form = form.text("x-goog-credential", parameters[5]["value"].as_str().unwrap().to_string());
        form = form.text("x-goog-algorithm", parameters[6]["value"].as_str().unwrap().to_string());
        form = form.text("x-goog-date", parameters[4]["value"].as_str().unwrap().to_string());
        form = form.text("x-goog-signature", parameters[7]["value"].as_str().unwrap().to_string());
        form = form.text("policy", parameters[8]["value"].as_str().unwrap().to_string());
        form = form.text("acl", parameters[2]["value"].as_str().unwrap().to_string());
        form = form.text("Content-Type", parameters[0]["value"].as_str().unwrap().to_string());
        form = form.text("success_action_status", parameters[1]["value"].as_str().unwrap().to_string());        
        form = form.part("file", some_file);
        let res2 = client
            .post(url)
            .multipart(form)
            .send()
            .await;
        if res2.is_err() {
          println!("Reqwest error uploading file form");
          return Err(io::Error::new(io::ErrorKind::Other, "Reqwest error uploading file form"));
        }
        let res2 = res2.unwrap();

        let result_xml = res2.text().await;
        if result_xml.is_err() {
          println!("Error reading result xml");
          return Err(io::Error::new(io::ErrorKind::Other, "Error reading result xml"));
        }
        let result_xml = result_xml.unwrap();

        let k = result_xml.split("<Key>");
        let fullKey = k.collect::<Vec<&str>>()[1];

        let k = fullKey.split("</Key>");
        let fullKey = k.collect::<Vec<&str>>()[0];

        // sleep for 5 seconds
        // sleep(Duration::from_secs(5)).await;

        println!("Upload complete");

        return Ok(fullKey.to_string());
    }
    Err(io::Error::new(io::ErrorKind::Other, "Error uploading"))
    
}