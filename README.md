# Proxy Checker

Simply tries proxy servers (http, https, socks4, socks5) and prints live ones

## Build

```cargo build --release```  

or statically:

```RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu```  

**NOTE**: because of a strange bug of cargo build you need explicitly specify target to use `RUSTFLAGS`  

## Usage

By default it reads from `stdin`:

```bash
cat <<EOF | ./proxy-checker -v -t 5 -r 3
1.2.3.4:80
5.6.7.8:90
9.10.11.12:100
EOF 
```

proxies can be read from a text file:

```bash
proxy-checker -t 5 -r 3 -f proxies.txt
```

**NOTE:** currently this doesn't check for different proxy types with each address and only uses the provided type from input
