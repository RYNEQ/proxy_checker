use std::error::Error;
use std::time::Duration;
use std::io::{self, BufRead};
use clap::Parser;


#[derive(Parser, Debug)]
#[clap(name = "Proxy Checker", author = "Ariyan Eghbal <ariyan.eghbal@gmail.com>", version = "0.1.0", about = "Checks if proxyies work", long_about = None)]
struct Args {
    #[clap(short = 'v', long= "verbose")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();


    let mut proxies = vec![];
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        proxies.push(line.expect("Could not read line from standard in"));
    }

    for p in proxies{
        if let Ok(proxy) = reqwest::Proxy::all(&p) { 
            if let Ok(client) = reqwest::Client::builder().danger_accept_invalid_certs(true).proxy(proxy).build() { 
                let resp = client.get("http://ifconfig.io/ip")
                    .header("Accept", "text/plain")
                    .header("User-Agent", "TEST")
                    .timeout(Duration::from_secs(10))
                    .send()
                    .await;
                if let Ok(resp) = resp {
                    match resp.text().await {
                        Ok(_) => println!("{}", p),
                        Err(e) => {
                            if args.verbose {
                                eprintln!("{}: {}", p, e.to_string()) ;
                            }
                            continue;
                        }
                    }
                }else if let Err(e) = resp {
                    if e.is_timeout() {
                        if args.verbose {
                            eprintln!("{}: Timeout", p);
                        }
                    } else {
                        if args.verbose {
                            eprintln!("{}: {}", p, e.source().unwrap().to_string());
                        }
                        continue;
                    }
                }
            } else {
                if args.verbose {
                    eprintln!("{} is not a valid proxy", p); 
                }
                continue;
            }
        }else{
            if args.verbose {
                eprintln!("Cannot parse {}", p); 
            }
            continue
        }
        
    }
    Ok(())
}
