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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    ClientMismatch,
    AlreadyDisputed,
    DisputeNotOpen,
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

    fn dispute(&self, account_store: &AccountStore) -> Result<(), ProcessError> {
        let result = account_store.modify(self.client, &|account: &mut Account| {
            let available = match account.available.checked_sub(self.amount) {
                None => return ProcessError::UnderflowError,
                Some(v) => v,
            };
            let held = match account.held.checked_add(self.amount) {
                None => return ProcessError::OverflowError,
                Some(v) => v,
            };
            account.available = available;
            account.held = held;
            ProcessError::Ok
        });

        match result {
            Ok(ProcessError::Ok) => Ok(()),
            Ok(err) => Err(err),
            Err(err) => Err(ProcessError::AccountError(err)),
        }
    }

    fn resolve(&self, account_store: &AccountStore) -> Result<(), ProcessError> {
        let result = account_store.modify(self.client, &|account: &mut Account| {
            let available = match account.available.checked_add(self.amount) {
                None => return ProcessError::OverflowError,
                Some(v) => v,
            };
            let held = match account.held.checked_sub(self.amount) {
                None => return ProcessError::UnderflowError,
                Some(v) => v,
            };
            // Only modify account at an infallible stage
            account.available = available;
            account.held = held;
            ProcessError::Ok
        });

        match result {
            Ok(ProcessError::Ok) => Ok(()),
            Ok(err) => Err(err),
            Err(err) => Err(ProcessError::AccountError(err)),
        }
    }

    fn chargeback(&self, account_store: &AccountStore) -> Result<(), ProcessError> {
        let result = account_store.modify(self.client, &|account: &mut Account| {
            account.held = match account.held.checked_sub(self.amount) {
                None => return ProcessError::UnderflowError,
                Some(v) => v,
            };
            account.frozen = true;
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
            (Vacant(_entry), _) => Err(ProcessError::TransactionNotFound),
            (Occupied(_entry), TransactionType::Deposit) => Err(ProcessError::TransactionExist),
            (Occupied(_entry), TransactionType::Withdrawal) => Err(ProcessError::TransactionExist),
            (Occupied(entry), TransactionType::Disputed) => {
                let trans = entry.into_mut();
                if trans.client != transaction.client {
                    return Err(ProcessError::ClientMismatch);
                }
                match trans.state {
                    TransactionState::Deposit => (),
                    TransactionState::Withdrawal => (),
                    _ => return Err(ProcessError::AlreadyDisputed),
                }
                trans.dispute(account_store)?;
                trans.state = TransactionState::Disputed;
                Ok(())
            }
            (Occupied(entry), TransactionType::Resolved) => {
                let trans = entry.into_mut();
                if trans.client != transaction.client {
                    return Err(ProcessError::ClientMismatch);
                }
                if trans.state != TransactionState::Disputed {
                    return Err(ProcessError::DisputeNotOpen);
                }
                trans.resolve(account_store)?;
                trans.state = TransactionState::Resolved;
                Ok(())
            }
            (Occupied(entry), TransactionType::Chargeback) => {
                let trans = entry.into_mut();
                if trans.client != transaction.client {
                    return Err(ProcessError::ClientMismatch);
                }
                if trans.state != TransactionState::Disputed {
                    return Err(ProcessError::DisputeNotOpen);
                }
                trans.chargeback(account_store)?;
                trans.state = TransactionState::Chargeback;
                Ok(())
            }
        };
        result
    }
}
