use std::io::{self, BufRead, Read};
use std::{error::Error,slice::Iter,fs};
use std::time::Duration;
use clap::Parser;

const COMMON_PROXY_PORTS:[&str;11] = ["1080","8080","9050","9051","8118","8123","8388","8580","8997","8998","8999"];

#[derive(Parser, Debug)]
#[clap(name = "Proxy Checker", author = "Ariyan Eghbal <ariyan.eghbal@gmail.com>", version = "0.4.0", about = "Checks if proxies work", long_about = None)]
struct Args {
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
    #[clap(short = 'q', long = "quiet", help = "If the quiet mode be on, program only show the live proxies")]
    quiet: bool,
    #[clap(short = 't', long = "timeout", default_value = "5")]
    timeout: u8,
    #[clap(short = 'T', long = "target", default_value = "https://www.google.com")]
    target_site: String,
    #[clap(short = 's', long = "string", help="String to search for in the target site")]
    check_str: Option<String>,
    #[clap(short = 'f', long = "file", help = "File containing proxies, one per line (without this proxies are read from stdin)")]
    proxy_file: Option<String>,
    #[clap(short = 'r', long = "repeat", default_value = "5", help = "Number of times to repeat the test for each proxy")]
    repeat: u8,
}

/// Takes a proxy string and returns makes error if it fails to connect to target
/// Returns Result<(), reqwest::Error>
async fn check_proxy(p: &String, timeout: u8, target: &String, test_string: &Option<String>) -> Result<bool, reqwest::Error> {
        let proxy = reqwest::Proxy::all(p)?; 
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .proxy(proxy)
            .build()?;
        let res = client.get(target)
                 .header("Accept", "text/plain")
                 .header("User-Agent", "TEST")
                 .timeout(Duration::from_secs(timeout as u64))
                 .send()
                 .await?
                 .text()
                 .await?;
        match test_string {
            Some(s) => {
                if res.contains(s) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            None => Ok(true),
        }
}

#[derive(Debug, PartialEq)]
enum ProxyTestResult {
    Success,
    Timeout,
    TextNotFound,
    Failure(String),
}

#[derive(Debug)]
enum Scheme {
    Http,
    Https,
    Socks4,
    Socks5,
}


impl Scheme {
    fn iter() -> Iter<'static, Scheme> {
        static SCHEMES: [Scheme; 4] = [Scheme::Http, 
                                       Scheme::Https, 
                                       Scheme::Socks4, 
                                       Scheme::Socks5];
        SCHEMES.iter()
    }

    fn value(&self) -> &str {
        match *self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Socks4 => "socks4",
            Scheme::Socks5 => "socks5"
        }
    }
}
fn get_url_without_scheme(url: &String) -> String {
    let url_without_scheme = url.split("://").last().unwrap();
    url_without_scheme.to_string()
}

fn is_addr_has_port(url: &String) -> bool{
    if url.contains(":"){
        let splitted = url.split(":").last().unwrap();
        if splitted.chars().all(|c| c.is_numeric()){
            true
        }else{
            false
        }
    }else{
        false
    }
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
        if !args.quiet{
            println!("Please enter your proxy address, One at each line, Enter empty line when finished:\n(If no port specified program will test some common ports on that address)\n");
        }
        
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            // Get entered addr
            let line_value = line.expect("Could not read line from standard in");
            if line_value == "" || line_value == "\n"{
                break
            }
            // The entered addr has port, just push into the proxies list
            if is_addr_has_port(&line_value){
                proxies.push(line_value)
            }else{
                // The entered addr hasn't specified port, so we select some common ports for that addr
                // Add url with some common ports
                for item in COMMON_PROXY_PORTS{
                    if line_value.ends_with(":"){
                        proxies.push(format!("{}{}",line_value,item));
                    }else{
                        proxies.push(format!("{}:{}",line_value,item));
                    }
                }
            }
        }
    }

    if !args.quiet{
        println!("Testing...\n");
    }

    let tasks = proxies.into_iter().map(|p| {
        let target = args.target_site.clone();
        let check_str = args.check_str.clone();
        tokio::spawn(async move{
            let mut result_list = vec![];
            for scheme in Scheme::iter() {
                let mut success_count = 0;
                let p_with_scheme = format!("{}://{}", scheme.value(), get_url_without_scheme(&p));
                for _ in 0..args.repeat {
                    let res = check_proxy(&p_with_scheme, args.timeout, &target, &check_str).await;
                    match res {
                        Ok(res) => {
                            if res {
                                success_count += 1;
                                result_list.push((ProxyTestResult::Success, scheme));
                            }else{
                                result_list.push((ProxyTestResult::TextNotFound, scheme));
                            }
                        },
                        Err(e) => {
                            if e.is_timeout(){ 
                                result_list.push((ProxyTestResult::Timeout, scheme));
                            }else{
                                result_list.push((ProxyTestResult::Failure(e.source().unwrap().to_string()), scheme));
                            }
                        }
                    }
                }
                if args.quiet{
                    if success_count > 0{
                        println!("{}",p_with_scheme);
                    }
                }else{
                    if args.verbose {
                        if success_count == 0{
                            println!("{}: Not Worked",p_with_scheme);
                        }else{
                            println!("{}: {}/{} Worked",p_with_scheme,success_count,args.repeat);
                        }
                        
                    }else {
                        if success_count > 0{
                            println!("{}: Worked", p_with_scheme);
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
