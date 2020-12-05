extern crate reqwest; // 0.9.18
extern crate serde_json;
extern crate serde;

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::fs::File;
use std::str::FromStr;

use chrono::{NaiveDate};

use serde::{Serialize, Deserialize};

macro_rules! TIME_SERIES_DAILY_ENDPOINT { () => { "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY_ADJUSTED&symbol={}&apikey=xxx" }; }

#[derive(Debug, Serialize, Deserialize)]
struct Stock {
    symbol: String, // PETR4, OIBR3, etc...
    start_date: NaiveDate,
    end_date: NaiveDate,
    price: Vec<i16>, // closing prices adjusted for splits
    volume: Vec<i32>, // in millions
}

fn download_symbol_data(symbol: String) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!(TIME_SERIES_DAILY_ENDPOINT!(), symbol);
    let mut res = reqwest::get(&endpoint)?; // TODO: actually check this error

    // TODO: how to handle API error without sadness?
    let root: HashMap<String, serde_json::Value> = serde_json::from_str(&res.text()?)?;
    let metadata: &serde_json::Map<String, serde_json::Value> = root.get("Meta Data").unwrap().as_object().unwrap();
    let time_series: &serde_json::Map<String, serde_json::Value> = root.get("Time Series (Daily)").unwrap().as_object().unwrap();

    let symbol: String = metadata.get("2. Symbol").unwrap().as_str().unwrap().to_string();
    let end_date: NaiveDate = NaiveDate::parse_from_str(metadata.get("3. Last Refreshed").unwrap().as_str().unwrap(), "%Y-%m-%d")?;

    let mut stock = Stock {
        symbol: symbol.clone(),
        start_date: NaiveDate::from_ymd(1800, 1, 1),
        end_date,
        price: Vec::with_capacity(365*20),
        volume: Vec::with_capacity(365*20),
    };

    let mut last_date = stock.start_date;
    let mut first = true;
    for (key, value) in time_series.into_iter() {
        let date = NaiveDate::parse_from_str(key, "%Y-%m-%d")?;
        if first {
            last_date = date.pred();
            stock.start_date = date;
            first = false;
        }
        assert!(date > last_date);

        let stock_info_map: &serde_json::Map<String, serde_json::Value> = value.as_object().unwrap();
        
        if stock.price.len() > 0 {
            for _ in 1..(date-last_date).num_days() {
                stock.price.push(stock.price[stock.price.len()-1]);
                stock.volume.push(stock.volume[stock.volume.len()-1]);
            }
        }

        let price = f32::from_str(stock_info_map.get("5. adjusted close").unwrap().as_str().unwrap())?;
        stock.price.push((price * 100.0) as i16);

        let volume = i64::from_str(stock_info_map.get("6. volume").unwrap().as_str().unwrap())?;
        stock.volume.push((volume / 1_000_000) as i32);
        
        last_date = date;
    }
    assert!(stock.start_date != NaiveDate::from_ymd(1800, 1, 1));
    
    let mut file = File::create(format!("symbols/{}.daily", symbol))?;
    file.write_all(serde_json::to_string(&stock).unwrap().as_bytes())?;
    
    Ok(())
}

// TODO: understand the box dyn thing
fn send_get() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = reqwest::get("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=PETR4.SA&apikey=xxx")?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&res.text()?)?;

    let metadata = map.get("Meta Data").unwrap();
    let data = map.get("Time Series (Daily)").unwrap();

    println!("{}", metadata);
    println!("{}", data);

    Ok(())
}

fn menu(money: &mut i64) {
    println!("Choose one:");
    println!("0- Check account balance");
    println!("1- Deposit money");
    println!("2- Withdraw money");
    println!("3- Buy a stock (NOT IMPLEMENTED)");
    println!("9- Dowload symbol data (DEBUG)");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Invalid input");

    let option: i32 = input.trim().parse().unwrap();

    match option {
        0 => println!("Account balance: {}", money),
        1 => {
            print!("How much do you wish to deposit? ");
            io::stdout().flush().unwrap();

            input.clear();
            io::stdin().read_line(&mut input).expect("Invalid input");

            *money += input.trim().parse::<i64>().unwrap();

            println!("New account balance: {}", money);
        },
        2 => {
            print!("How much do you wish to withdraw? ");
            io::stdout().flush().unwrap();

            input.clear();
            io::stdin().read_line(&mut input).expect("Invalid input");

            *money -= input.trim().parse::<i64>().unwrap();

            println!("New account balance: {}", money);
        },
        3 => send_get().unwrap(),
        9 => {
            print!("Which symbol? ");
            io::stdout().flush().unwrap();

            input.clear();
            io::stdin().read_line(&mut input).expect("Invalid input");

            input = format!("{}{}", input.trim(), ".SA");

            // TODO: check for error (invalid stock)
            download_symbol_data(input.to_string()).unwrap();
        },
        _ => ()
    }
    println!("");
}

fn main() {
    let mut money: i64 = 0;
    loop {
        menu(&mut money);
    }
}
