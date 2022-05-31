use std::error::Error;
use std::time::Duration;
use std::io::{self, BufRead};
use clap::Parser;


#[derive(Parser, Debug)]
#[clap(name = "Proxy Checker", author = "Ariyan Eghbal <ariyan.eghbal@gmail.com>", version = "0.1.0", about = "Checks if proxyies work", long_about = None)]
struct Args {
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
    #[clap(short = 't', long = "timeout", default_value = "5")]
    timeout: u8,
}


async fn check_proxy(p: &String, timeout: u8) -> Result<(), reqwest::Error> {
        let proxy = reqwest::Proxy::all(p)?; 
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(proxy)
            .build()?;
        client.get("http://ifconfig.io/ip")
            .header("Accept", "text/plain")
            .header("User-Agent", "TEST")
            .timeout(Duration::from_secs(timeout as u64))
            .send()
            .await?
            .text()
            .await?; 
    Ok(())
}


#[tokio::main]
async fn main() {
    let args = Args::parse();


    let mut proxies = vec![];
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        proxies.push(line.expect("Could not read line from standard in"));
    }

    let tasks = proxies.into_iter().map(|p| {
        tokio::spawn(async move{
            let res = check_proxy(&p, args.timeout).await;
            match res {
                Ok(_) => {
                    println!("{}", p);
                },
                Err(e) => {
                    if args.verbose {
                        if e.is_timeout(){ 
                            println!("{}: Timeout", p);
                        }else{
                            println!("{}: {}", p, e.source().unwrap().to_string());
                        }
                    }
                }
            }
        })
    }).collect::<Vec<_>>();

    for task in tasks {
        task.await.unwrap();
    }
}
