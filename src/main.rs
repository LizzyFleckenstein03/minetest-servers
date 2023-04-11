use anyhow::anyhow;
use clap::CommandFactory;
use clap::Parser;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

fn main() -> anyhow::Result<()> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        /// Address of the server list
        #[clap(short, long, value_parser, default_value_t = String::from("https://servers.minetest.net/list"))]
        address: String,

        /// List available keys
        #[clap(short, long, action)]
        show_keys: bool,

        /// The key to look up. Use --show-keys to list available keys.
        key: Option<String>,
    }
    let args = Args::parse();

    #[derive(Deserialize)]
    struct Payload {
        list: Vec<BTreeMap<String, JsonValue>>,
    }
    let payload: Payload = reqwest::blocking::get(args.address)?.json()?;

    if args.show_keys {
        let keys: BTreeSet<&String> = payload.list.iter().flat_map(BTreeMap::keys).collect();
        println!("available keys: {}", keys.iter().format(", "));
    }

    if let Some(key) = args.key {
        let mut iter = payload
            .list
            .iter()
            .flat_map(|server| server.get(&key).map(|value| (server, value)))
            .peekable();

        if iter.peek().is_none() {
            return Err(anyhow!("invalid key"));
        } else {
            iter.for_each(|(server, value)| {
                struct DisplayJson<'a>(Option<&'a JsonValue>);
                impl<'a> fmt::Display for DisplayJson<'a> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        match self.0 {
                            None => Ok(()),
                            Some(value) => match value {
                                JsonValue::String(x) => write!(f, "{x}"),
                                _ => write!(f, "{value}"),
                            },
                        }
                    }
                }

                let address = DisplayJson(server.get("address"));
                let port = DisplayJson(server.get("port"));

                if let JsonValue::Array(array) = value {
                    let prefix = format!("{address}:{port}"); // cache it
                    for value in array.iter() {
                        println!("{prefix}\t{}", DisplayJson(Some(value)));
                    }
                } else {
                    println!("{address}:{port}\t{}", DisplayJson(Some(value)));
                }
            });
        }
    } else if !args.show_keys {
        Args::command().print_help()?;
    }

    Ok(())
}
