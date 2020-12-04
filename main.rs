extern crate reqwest; // 0.9.18
extern crate serde_json;

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::fs::File;

macro_rules! TIME_SERIES_DAILY_ENDPOINT { () => { "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol={}&apikey=xxx" }; }

fn download_symbol_data(symbol: String) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!(TIME_SERIES_DAILY_ENDPOINT!(), symbol);
    let mut res = reqwest::get(&endpoint)?; // TODO: actually check this error
    let mut file = File::create(format!("symbols/{}.daily", symbol))?;
    file.write_all(res.text()?.as_bytes())?;
    
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
    println!("9- Dowload symbol data");

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
            input.clear();
            io::stdin().read_line(&mut input).expect("Invalid input");

            download_symbol_data(input.trim().to_string()).unwrap();
        },
        _ => ()
    }
    println!("");
}

fn main() {
    print!("How much money is in your balance? ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Invalid input");

    let mut money: i64 = input.trim().parse().unwrap();

    loop {
        menu(&mut money);
    }
}
