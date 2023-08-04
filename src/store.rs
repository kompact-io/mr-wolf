use cryptoxide::ed25519;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::types::{mk_message, Nonce, PubKey, Sig, Witness};

#[derive(Debug, Clone)]
pub struct Account {
    pub pub_key: PubKey,
    pub pot: u64,     // Cumulative total in pot
    pub used: u64,    // Cumulative used so far
    pub iou: u64,     // highest code. Should be that: used <= code <= pot
    pub nonce: Nonce, // nonce for code
    pub sig: Sig,     // proof of code
}

impl Account {
    pub fn new(pub_key: PubKey, pot: u64) -> Self {
        Account {
            pub_key,
            pot,
            used: 0,
            iou: 0,
            nonce: [0; 16],
            sig: [0; 64],
        }
    }
}

#[derive(Debug, Default)]
pub struct Store {
    shared: Arc<Shared>,
}

#[derive(Debug, Default)]
struct Shared {
    accounts: Mutex<BTreeMap<Uuid, Account>>,
}

#[derive(Debug)]
pub enum StoreError {
    KeyDoesNotExist,
    BadWitness,
    IouNotCovered,
}

type StoreResult<T> = Result<T, StoreError>;

impl Store {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                accounts: Mutex::new(BTreeMap::new()),
            }),
        }
    }
    pub fn add(&self, pub_key: PubKey, pot: u64) -> StoreResult<Uuid> {
        let id = Uuid::new_v4();
        let account = Account::new(pub_key, pot);
        let mut accounts = self.shared.accounts.lock().unwrap();
        accounts.insert(id, account);
        Ok(id)
    }
    pub fn get(&self, id: Uuid) -> StoreResult<Account> {
        let accounts = self.shared.accounts.lock().unwrap();
        let entry = accounts
            .get(&id)
            .ok_or(StoreError::KeyDoesNotExist)?
            .clone();
        Ok(entry)
    }
    // pub fn inc_pot(&self, id: Uuid, inc: u64) -> Result<(), StoreError> {
    //     let mut accounts = self.shared.accounts.lock().unwrap();
    //     let curr = accounts
    //         .get(&id)
    //         .ok_or(StoreError::KeyDoesNotExist)?
    //         .clone();
    //     accounts.insert(
    //         id,
    //         Account {
    //             pot: curr.pot + inc,
    //             ..curr
    //         },
    //     );
    //     Ok(())
    // }

    pub fn update_balance(&self, id: Uuid, witness: Witness) -> StoreResult<u64> {
        let mut accounts = self.shared.accounts.lock().unwrap();
        let curr = accounts
            .get(&id)
            .ok_or(StoreError::KeyDoesNotExist)?
            .clone();
        if witness.iou <= curr.iou {
            return Ok(curr.iou - curr.used);
        }
        if witness.iou > curr.pot {
            return Err(StoreError::IouNotCovered);
        }
        let message = mk_message(witness.iou, &witness.nonce);
        let is_valid = ed25519::verify(&message, &curr.pub_key, &witness.sig);
        if !is_valid {
            return Err(StoreError::BadWitness);
        }
        let balance = witness.iou - curr.used;
        accounts.insert(
            id,
            Account {
                iou: witness.iou,
                nonce: witness.nonce,
                sig: witness.sig,
                ..curr
            },
        );
        Ok(balance)
    }
    pub fn inc_used(&self, id: Uuid, cost: u64) -> StoreResult<()> {
        let mut accounts = self.shared.accounts.lock().unwrap();
        let curr = accounts
            .get(&id)
            .ok_or(StoreError::KeyDoesNotExist)?
            .clone();
        accounts.insert(
            id,
            Account {
                used: curr.used + cost,
                ..curr
            },
        );
        Ok(())
    }
}
