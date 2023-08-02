mod utils;

use std::collections::HashSet;

use reqwest::Response;
use serde_json::to_string;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use reqwest::Client;
use reqwest::Error;
use reqwest::StatusCode;
use serde_json::Value;
use serde_json::json;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
struct HitsData {
    took: u32,
    timed_out: bool,
    _shards: Shards,
    hits: Hits,
}

#[derive(Debug, Deserialize, Serialize)]
struct Shards {
    total: u32,
    successful: u32,
    skipped: u32,
    failed: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Hits {
    total: HitsTotal,
    max_score: f64,
    hits: Vec<Hit>,
}

#[derive(Debug, Deserialize, Serialize)]
struct HitsTotal {
    value: u32,
    relation: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Hit {
    _index: String,
    _type: String,
    _id: String,
    _score: f64,
    _source: Source,
}

#[derive(Debug, Deserialize, Serialize)]
struct Source {
    number_of_veg: Option<String>,
    species: Option<String>,
    date_picked: Option<String>,
    weight: Option<String>,
    vegetable: Option<String>,
}


#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-farm!");
}

#[wasm_bindgen]
pub async fn add_to_db(veg_name: String, species: String, nr: String, weight: String, date_picked: String) -> String {
    match send_data_to_elasticsearch(&veg_name.as_str(), &species.as_str() , &nr.as_str(), &weight.as_str(), &date_picked.as_str()).await {
        Ok(result) => result.text().await.unwrap(),
        Err(_) => "Shit happened".to_string()
    }
}

async fn send_data_to_elasticsearch(veg: &str, species: &str, nr_of_veg: &str, weight: &str, date_picked: &str) -> Result<Response, Error> {
    let client = Client::new();

    let url = "http://192.168.0.250:9234/horta_do_tarrafal/_doc";
    // Replace <elasticsearch_server>, <port>, and <index> with the appropriate values
    let json_data = json!({ "vegetable": veg, "species": species, "number_of_veg":  nr_of_veg, "weight": weight, "date_picked": date_picked});
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(json_data.to_string())
        .send()
        .await?;

    if response.status().is_success() {
        println!("Data sent successfully!");
    } else {
        println!("Failed to send data. Status code: {}", response.status());
    }

    Ok(response)
}

#[wasm_bindgen]
pub async fn get_db_data() -> JsValue {
    let big_vec = query_all_data().await;
    serde_wasm_bindgen::to_value(&big_vec).unwrap()
}

async fn get_data_from_elasticsearch() -> Result<Response, Error> {
    
    // Define the Elasticsearch endpoint URL
    let url = "http://192.168.0.250:9234/horta_do_tarrafal/_search";

    // Create a reqwest client
    let client = Client::new();

    // Create the Elasticsearch query to retrieve all documents and only the "field_name" field
    let query = json!({
        "query": {
            "match_all": {}
        },
        "_source": ["vegetable", "species", "number_of_veg", "weight", "date_picked"],
        "size": 1000 // Set the size to the number of documents you want to retrieve (e.g., 1000)
    });

    let response = client.post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(query.to_string())
        .send()
        .await?;

    if response.status().is_success() {
        println!("Data received successfully!");
    } else {
        println!("Failed to get data. Status code: {}", response.status());
    }

    Ok(response)

}

async fn query_all_data() -> Vec<Vec<String>>  {
    let jsonstring = match get_data_from_elasticsearch().await {
        Ok(result) => result.text().await.unwrap(),
        Err(_) => "Shit happened".to_string()
    };

    let parsed_data: HitsData  = serde_json::from_str(jsonstring.as_str()).expect("Failed to parse JSON");

    let mut big_vec: Vec<Vec<String>> = Vec::new();
    let mut small_vec: Vec<String> = Vec::new();

    for hit in parsed_data.hits.hits {
        if let Some(vegetable) = hit._source.vegetable {
            small_vec.push(vegetable);
        }
        if let Some(species) = hit._source.species {
            small_vec.push(species);
        }
        if let Some(number_of_veg) = hit._source.number_of_veg {
            small_vec.push(number_of_veg);
        }
        if let Some(weight) = hit._source.weight {
            small_vec.push(weight);
        }
        if let Some(date_picked) = hit._source.date_picked {
            small_vec.push(date_picked);          
        }
        big_vec.push(small_vec.clone());
        small_vec.clear();
    }
    big_vec.sort_by(|a, b| a[4].cmp(&b[4]));
    big_vec.reverse();
    big_vec
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct Statistics {
    name: String,
    accum_amount: u64,
    accum_weight: u64
}

// Implementing the ToString trait for MyStruct
impl std::string::ToString for Statistics {
    fn to_string(&self) -> String {
        format!("{}, amount: {} , weight: {}g\n", self.name, self.accum_amount, self.accum_weight)
    }
}

#[wasm_bindgen]
pub async fn stattest() -> String {
    let test = sum_weight_amount().await;
    let mut my_string = String::from("");
    for elem in &test {
        my_string.push_str(elem.to_string().as_str());
    }

    my_string
}

async fn sum_weight_amount() -> Vec<Statistics> {
    let big_vec: Vec<Vec<String>> = query_all_data().await;
    // let mut stat_hash: HashSet<Statistics> = HashSet::new();

    let mut new_vec: Vec<Statistics> = Vec::new();

    for item in big_vec {
        let target_string = item[0].clone();

        let result = new_vec.iter_mut().find(|stat| stat.name == target_string);

        match result {
            Some(found_struct) => {
                found_struct.accum_amount += item[2].parse::<u64>().unwrap();
                found_struct.accum_weight += item[3].parse::<u64>().unwrap();
                println!("Found struct with name '{}', amount: {}, weight: {}", found_struct.name, found_struct.accum_amount, found_struct.accum_weight);
            }
            None => {
                new_vec.push(Statistics {
                    name: target_string,
                    accum_amount: item[2].parse::<u64>().unwrap(),
                    accum_weight:item[3].parse::<u64>().unwrap()
                });
                println!("No struct found with name '{}'", item[0].clone());
            }
        }
    }

    new_vec
}