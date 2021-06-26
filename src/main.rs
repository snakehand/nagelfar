mod account;
mod account_transactions;
mod amount;

use account::{Account, AccountId, AccountStore};
use account_transactions::{AccountTransaction, TransactionId, TransactionStore, TransactionType};
use amount::Amount;

fn main() {
    let a1: Account = Default::default();
    let s: f64 = a1
        .total()
        .unwrap()
        .checked_sub(Amount::new(123.023999).unwrap())
        .unwrap()
        .into();
    println!("{}", s);

    let mut account_store = AccountStore::new();
    let mut transactions = TransactionStore::new();

    let client = AccountId::new(42);
    let tx_id = TransactionId::new(1103);
    let ttype = TransactionType::Deposit;
    let amount = Amount::new(2000.4999).unwrap();
    let mut trans = AccountTransaction {
        client,
        tx_id,
        ttype,
        amount,
    };
    let res = transactions.process_transaction(trans, &account_store);
    println!("{:?}", res);
    let res = transactions.process_transaction(trans, &account_store);
    println!("{:?}", res);
    trans.ttype = TransactionType::Withdrawal;
    trans.amount = Amount::new(3000.4999).unwrap();
    trans.tx_id = TransactionId::new(1104);
    let res = transactions.process_transaction(trans, &account_store);
    println!("{:?}", res);

    println!("{:?}", account_store);
}
