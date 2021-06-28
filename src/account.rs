use crate::amount::Amount;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct AccountId(u16);

impl AccountId {
    pub fn new(id: u16) -> Self {
        AccountId(id)
    }
}

#[derive(Debug, Default)]
pub struct Account {
    pub available: Amount,
    pub held: Amount,
    pub frozen: bool,
}

impl Account {
    // Make accessor to preserve invariant
    pub fn total(&self) -> Option<Amount> {
        self.available.checked_add(self.held)
    }
}

#[derive(Debug)]
pub struct AccountStore {
    store: Mutex<HashMap<AccountId, Account>>,
}

#[derive(Debug, Copy, Clone)]
pub enum AccountModifyError {
    // NotFound,          // Acount could not be found, or creation failed
    Frozen,            // Account is locked and can not be modified
    TransactionFailed, // Atomic operation failed
}

impl AccountStore {
    pub fn new() -> Self {
        let store = Mutex::new(HashMap::new());
        AccountStore { store }
    }

    pub fn modify<T>(
        &self,
        id: AccountId,
        proc: &impl Fn(&mut Account) -> T,
    ) -> Result<T, AccountModifyError> {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(_) => return Err(AccountModifyError::TransactionFailed), // Poisoned mutex
        };
        let account = match store.entry(id) {
            Vacant(entry) => entry.insert(Default::default()),
            Occupied(entry) => entry.into_mut(),
        };
        if account.frozen {
            return Err(AccountModifyError::Frozen);
        }
        let res = proc(account);
        Ok(res)
    }

    pub fn modify_force<T>(
        &self,
        id: AccountId,
        proc: &impl Fn(&mut Account) -> T,
    ) -> Result<T, AccountModifyError> {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(_) => return Err(AccountModifyError::TransactionFailed), // Poisoned mutex
        };
        let account = match store.entry(id) {
            Vacant(entry) => entry.insert(Default::default()),
            Occupied(entry) => entry.into_mut(),
        };
        let res = proc(account);
        Ok(res)
    }

    pub fn print(&self) {
        println!("client, available, held, total, locked");
        let store = self.store.lock().unwrap();
        for (cid, account) in store.iter() {
            println!(
                "{}, {:.4}, {:.4}, {:.4}, {}",
                cid.0,
                f64::from(account.available),
                f64::from(account.held),
                f64::from(account.total().unwrap()),
                account.frozen
            );
        }
    }
}
