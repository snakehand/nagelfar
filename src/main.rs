mod account;
mod account_transactions;
mod amount;

use account::{AccountId, AccountStore};
use account_transactions::{AccountTransaction, TransactionId, TransactionStore, TransactionType};
use amount::Amount;

fn main() {
    let account_store = AccountStore::new();
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

    for trans in transaction_list.iter() {
        let res = transactions.process_transaction(*trans, &account_store);
        println!("{:?}", res);
    }

    account_store.print();
}
