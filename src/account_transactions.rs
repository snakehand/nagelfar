use crate::account::{Account, AccountId, AccountModifyError, AccountStore};
use crate::amount::Amount;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct TransactionId(u32);

impl TransactionId {
    pub fn new(id: u32) -> Self {
        TransactionId(id)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Disputed,
    Resolved,
    Chargeback,
}

// Encodes transactions as the appear in the input file
#[derive(Debug, Copy, Clone)]
pub struct AccountTransaction {
    pub client: AccountId,
    pub tx_id: TransactionId,
    pub ttype: TransactionType,
    pub amount: Amount,
}

#[derive(Debug, Copy, Clone)]
pub enum TransactionState {
    Deposit,
    Withdrawal,
    Disputed,
    Resolved,
    Chargeback,
    FailedDeposit,
    FailedWithdrawal,
}

#[derive(Debug)]
pub struct Transaction {
    client: AccountId,
    state: TransactionState,
    amount: Amount,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessError {
    TransactionExist,
    TransactionNotFound,
    AccountError(AccountModifyError),
    OverflowError,
    InsufficientFunds,
    UnderflowError,
    Ok,
}

impl Transaction {
    fn new(ac_trans: &AccountTransaction) -> Self {
        let client = ac_trans.client;
        let state = match ac_trans.ttype {
            TransactionType::Deposit => TransactionState::Deposit,
            TransactionType::Withdrawal => TransactionState::Withdrawal,
            TransactionType::Disputed => TransactionState::Disputed,
            TransactionType::Resolved => TransactionState::Resolved,
            TransactionType::Chargeback => TransactionState::Chargeback,
        };
        let amount = ac_trans.amount;
        Transaction {
            client,
            state,
            amount,
        }
    }

    fn deposit(&self, account_store: &AccountStore) -> Result<(), ProcessError> {
        let result = account_store.modify(self.client, &|account: &mut Account| {
            account.available = match account.available.checked_add(self.amount) {
                None => return ProcessError::OverflowError,
                Some(v) => v,
            };
            ProcessError::Ok
        });

        match result {
            Ok(ProcessError::Ok) => Ok(()),
            Ok(err) => Err(err),
            Err(err) => Err(ProcessError::AccountError(err)),
        }
    }

    fn withdraw(&self, account_store: &AccountStore) -> Result<(), ProcessError> {
        let result = account_store.modify(self.client, &|account: &mut Account| {
            if self.amount > account.available {
                return ProcessError::InsufficientFunds;
            }
            account.available = match account.available.checked_sub(self.amount) {
                None => return ProcessError::UnderflowError,
                Some(v) => v,
            };
            ProcessError::Ok
        });

        match result {
            Ok(ProcessError::Ok) => Ok(()),
            Ok(err) => Err(err),
            Err(err) => Err(ProcessError::AccountError(err)),
        }
    }
}

// Sharded transaction storage - for simplicity this code operates with just a single shard
pub struct TransactionStore {
    store: HashMap<TransactionId, Transaction>,
}

impl TransactionStore {
    pub fn new() -> Self {
        let store = HashMap::new();
        TransactionStore { store }
    }

    pub fn process_transaction(
        &mut self,
        trans: AccountTransaction,
        account_store: &AccountStore,
    ) -> Result<(), ProcessError> {
        let transaction = Transaction::new(&trans);
        let result = match (self.store.entry(trans.tx_id), trans.ttype) {
            (Vacant(entry), TransactionType::Deposit) => {
                transaction.deposit(account_store)?;
                entry.insert(transaction);
                Ok(())
            }
            (Vacant(entry), TransactionType::Withdrawal) => {
                transaction.withdraw(account_store)?;
                entry.insert(transaction);
                Ok(())
            }
            (Vacant(entry), _) => Err(ProcessError::TransactionNotFound),
            (Occupied(entry), TransactionType::Deposit) => Err(ProcessError::TransactionExist),
            (Occupied(entry), TransactionType::Withdrawal) => Err(ProcessError::TransactionExist),
            (Occupied(entry), _) => {
                let t = entry.into_mut();
                Ok(())
            }
        };
        result
    }
}
