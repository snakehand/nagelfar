mod account;
mod account_transactions;
mod amount;

use account::{Account, AccountId, AccountStore};
use account_transactions::{AccountTransaction, TransactionId, TransactionStore, TransactionType};
use amount::Amount;

fn main() {
    let mut account_store = AccountStore::new();
    let mut transactions = TransactionStore::new();

    fn trans(cid: u16, tid: u32, ttype: TransactionType, amount: f64) -> AccountTransaction {
        let client = AccountId::new(cid);
        let tx_id = TransactionId::new(tid);
        let amount = Amount::new(amount).unwrap();
        AccountTransaction {
            client,
            tx_id,
            ttype,
            amount,
        }
    }

    let res = transactions.process_transaction(
        trans(42, 1103, TransactionType::Deposit, 2000.4999),
        &account_store,
    );
    println!("{:?}", res);

    let res = transactions.process_transaction(
        trans(42, 1104, TransactionType::Withdrawal, 3000.4999),
        &account_store,
    );
    println!("{:?}", res);

    let res = transactions.process_transaction(
        trans(42, 1103, TransactionType::Disputed, 3000.4999),
        &account_store,
    );
    println!("{:?}", res);

    let res = transactions.process_transaction(
        trans(42, 1105, TransactionType::Deposit, 355.8674),
        &account_store,
    );
    println!("{:?}", res);

    let res = transactions.process_transaction(
        trans(42, 1103, TransactionType::Chargeback, 3000.4999),
        &account_store,
    );
    println!("{:?}", res);

    let res = transactions.process_transaction(
        trans(42, 1106, TransactionType::Deposit, 355.8674),
        &account_store,
    );
    println!("{:?}", res);

    println!("{:?}", account_store);
}
