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
    #[clap(short = 'T', long = "target", default_value = "https://www.google.com")]
    target_site: String,
    #[clap(short = 'f', long = "file", help = "File containing proxies, one per line (without this proxies are read from stdin)")]
    proxy_file: Option<String>,
    #[clap(short = 'r', long = "repeat", default_value = "5", help = "Number of times to repeat the test for each proxy")]
    repeat: u8,
}


async fn check_proxy(p: &String, timeout: u8, target: &String) -> Result<(), reqwest::Error> {
        let proxy = reqwest::Proxy::all(p)?; 
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(proxy)
            .build()?;
        client.get(target)
            .header("Accept", "text/plain")
            .header("User-Agent", "TEST")
            .timeout(Duration::from_secs(timeout as u64))
            .send()
            .await?
            .text()
            .await?; 
    Ok(())
}

#[derive(Debug, PartialEq)]
enum ProxyTestResult {
    Success,
    Timeout,
    Failure(String),
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
            let mut success_count = 0;
            let mut result_list = vec![];
            for _ in 0..args.repeat {
                let res = check_proxy(&p, args.timeout, &target).await;
                match res {
                    Ok(_) => {
                        success_count += 1;
                        result_list.push(ProxyTestResult::Success);
                    },
                    Err(e) => {
                        if e.is_timeout(){ 
                            result_list.push(ProxyTestResult::Timeout);
                        }else{
                            result_list.push(ProxyTestResult::Failure(e.source().unwrap().to_string()));
                        }
                    }
                }
            }
            if args.verbose {
                if success_count == args.repeat {
                    println!("{}: Success", p);
                }else{
                    println!("{}: {}/{}", p, success_count, args.repeat);
                }
            }else{
                println!("{}", p);
            }
        })
    }).collect::<Vec<_>>();

    for task in tasks {
        task.await.unwrap();
    }
}
