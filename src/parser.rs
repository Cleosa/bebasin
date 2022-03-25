use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::prelude::*;

use itertools::Itertools as _;
use pest::Parser;

use crate::error::ErrorKind;

type Hosts = HashMap<String, HashSet<String>>;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct HostsParser;

pub fn write_to_file(file_path: &str, hosts: &Hosts, header: &str) -> Result<(), ErrorKind> {
    let mut file;
    match fs::File::create(file_path) {
        Ok(f) => {
            file = f;
        }
        Err(err) => {
            return Err(ErrorKind::IOError(err));
        }
    };

    let mut hosts_stringify = String::new();
    hosts_stringify.push_str(header);
    for host in hosts {
        let ip = &host.0;
        let hostnames = &host.1.into_iter().join(" ");

        hosts_stringify.push_str(&format!("{} {}\n", ip, hostnames));
    }

    match file.write_all(hosts_stringify.as_bytes().as_ref()) {
        Ok(()) => Ok(()),
        Err(err) => Err(ErrorKind::IOError(err)),
    }
}

pub fn parse_from_file(file_path: &str) -> Result<Hosts, ErrorKind> {
    match fs::read_to_string(file_path) {
        Ok(str) => parse_from_str(&str),
        Err(err) => Err(ErrorKind::IOError(err)),
    }
}

pub fn parse_from_str(str: &str) -> Result<Hosts, ErrorKind> {
    let mut hosts: Hosts = HashMap::new();
    let res = match HostsParser::parse(Rule::main, str) {
        Ok(x) => x,
        Err(err) => return Err(ErrorKind::PestRuleError(err)),
    };

    for pair in res {
        if let Rule::statement = pair.as_rule() {
            let mut ip = String::new();
            let mut hostnames: HashSet<String> = HashSet::new();

            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::ip => {
                        ip = inner_pair.as_str().to_owned();
                    }
                    Rule::hostnames => {
                        for hostname in inner_pair.into_inner() {
                            hostnames.insert(hostname.as_str().to_owned());
                        }
                    }
                    _ => {}
                }
            }

            match hosts.get_mut(&ip) {
                Some(old_val) => {
                    hostnames.into_iter().for_each(|x| {
                        old_val.insert(x);
                    });
                    // old_val.append(&mut hostnames);
                }
                None => {
                    hosts.insert(ip, hostnames);
                }
            };
        }
    }
    Ok(hosts)
}
