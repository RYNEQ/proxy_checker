use std::io::{self, BufRead, Read};
use std::{error::Error,fs};
use std::time::Duration;
use clap::Parser;


#[derive(Parser, Debug)]
#[clap(name = "Proxy Checker", author = "Ariyan Eghbal <ariyan.eghbal@gmail.com>", version = "0.2.0", about = "Checks if proxyies work", long_about = None)]
struct Args {
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
    #[clap(short = 't', long = "timeout", default_value = "5")]
    timeout: u8,
    #[clap(short = 't', long = "target", default_value = "https://www.google.com")]
    target_site: String,
    #[clap(short = 'f', long = "file")]
    proxy_file: Option<String>,
}


async fn check_proxy(p: &String, timeout: u8, target: String) -> Result<(), reqwest::Error> {
        let proxy = reqwest::Proxy::all(p)?; 
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(proxy)
            .build()?;
        client.get(&target)
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
    if args.proxy_file.is_some() {
        let file = args.proxy_file.unwrap();
        let mut f = fs::File::open(file).expect("File not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("Could not read file");
        proxies = contents.split("\n").map(|x| x.to_string()).collect();
    }else{
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            proxies.push(line.expect("Could not read line from standard in"));
        }
    }

    let tasks = proxies.into_iter().map(|p| {
        let target = args.target_site.clone();
        tokio::spawn(async move{
            let res = check_proxy(&p, args.timeout, target).await;
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
