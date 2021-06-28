mod account;
mod account_transactions;
mod amount;

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use account::{AccountId, AccountStore};
use account_transactions::{AccountTransaction, TransactionId, TransactionStore, TransactionType};
use amount::Amount;

struct InputFile {
    bufread: Box<dyn BufRead>,
}

fn trans(cid: u16, tid: u32, ttype: TransactionType, amount: f64) -> Option<AccountTransaction> {
    let client = AccountId::new(cid);
    let tx_id = TransactionId::new(tid);
    let amount = match Amount::new(amount) {
        Some(a) => a,
        None => return None,
    };
    Some(AccountTransaction {
        client,
        tx_id,
        ttype,
        amount,
    })
}

impl Iterator for InputFile {
    type Item = AccountTransaction;

    fn next(&mut self) -> Option<AccountTransaction> {
        loop {
            let mut line = String::new();
            match self.bufread.read_line(&mut line) {
                Err(_) => return None,
                Ok(0) => return None,
                _ => (),
            }
            // Assume comma is used as seperator
            let parts: Vec<&str> = line.split(",").map(|l| l.trim()).collect();
            if parts.len() < 3 {
                continue;
            }
            let ttype = match parts[0] {
                "deposit" => TransactionType::Deposit,
                "withdrawal" => TransactionType::Withdrawal,
                "dispute" => TransactionType::Disputed,
                "resolve" => TransactionType::Resolved,
                "chargeback" => TransactionType::Chargeback,
                _ => continue,
            };
            let client = match parts[1].parse::<u16>() {
                Ok(c) => c,
                Err(_) => continue,
            };
            let txid = match parts[2].parse::<u32>() {
                Ok(t) => t,
                Err(_) => continue,
            };
            let amount =
                if ttype == TransactionType::Deposit || ttype == TransactionType::Withdrawal {
                    if parts.len() < 4 {
                        continue;
                    }
                    match parts[3].parse::<f64>() {
                        Ok(a) => a,
                        Err(_) => continue,
                    }
                } else {
                    0.0 // Suupply dummy 0 dispute related
                };
            let trx = match trans(client, txid, ttype, amount) {
                Some(tx) => tx,
                None => continue,
            };
            return Some(trx);
        }
    }
}

fn read_file(filename: &Path) -> io::Result<impl Iterator<Item = AccountTransaction>> {
    let file = File::open(filename)?;
    let mut bufread = Box::new(io::BufReader::new(file));
    let mut discard = String::new();
    bufread.read_line(&mut discard)?; // Discard 1 line
    Ok(InputFile { bufread })
}

fn main() {
    let account_store = AccountStore::new();
    let mut transactions = TransactionStore::new();
    if env::args().len() < 2 {
        eprintln!("Filename not supplied.");
        return;
    }

    let args: Vec<OsString> = env::args_os().collect();
    let input = read_file(&PathBuf::from(args[1].clone())).unwrap();
    for trx in input {
        let _res = transactions.process_transaction(trx, &account_store);
        // println!("{:?}", _res);
    }

    /*
        let transaction_list = [
            trans(42, 1103, TransactionType::Deposit, 2000.4999),
            trans(42, 1104, TransactionType::Withdrawal, 3000.4999),
            trans(42, 1103, TransactionType::Disputed, 0.0),
            trans(42, 1105, TransactionType::Deposit, 355.8674),
            trans(42, 1103, TransactionType::Chargeback, 0.0),
            trans(42, 1106, TransactionType::Deposit, 355.8674),
            trans(42, 1103, TransactionType::Deposit, 2000.4999),
            trans(42, 1104, TransactionType::Withdrawal, 3000.4999),
            trans(42, 1103, TransactionType::Disputed, 0.0),
            trans(42, 1105, TransactionType::Deposit, 355.8674),
            trans(42, 1103, TransactionType::Chargeback, 0.0),
            trans(43, 1106, TransactionType::Deposit, 499.8123),
            trans(43, 1107, TransactionType::Deposit, 111.2314),
            trans(43, 1106, TransactionType::Disputed, 0.0),
            trans(43, 1106, TransactionType::Resolved, 0.0),
            trans(43, 1106, TransactionType::Disputed, 0.0),
            trans(43, 1106, TransactionType::Chargeback, 0.0),
        ];

        for trx in transaction_list.iter() {
            let res = transactions.process_transaction(trx.unwrap(), &account_store);
            println!("{:?}", res);
        }
    */

    account_store.print();
}
