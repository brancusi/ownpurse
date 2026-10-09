//! Test helper for the multi-process refresh race: `ownpurse-token-probe <token-file> <client-id> <token-url>`
//! prints the access token `TokenStore::access_token` returns (exit 0), or `error: <code>` (exit 1).
//! The token URL must be loopback: this binary never talks to Xero.

use ownpurse_xero::TokenStore;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let [path, client, url] = [&a[0], &a[1], &a[2]];
    let host = url
        .split("://")
        .nth(1)
        .unwrap_or("")
        .split(['/', ':'])
        .next()
        .unwrap_or("");
    assert!(
        host == "127.0.0.1" || host == "localhost",
        "token probe only talks to loopback, got {host}"
    );
    match TokenStore::new(path, client.as_str())
        .with_token_url(url.as_str())
        .access_token()
    {
        Ok(t) => println!("{t}"),
        Err(e) => {
            println!("error: {}", e.code());
            std::process::exit(1);
        }
    }
}
