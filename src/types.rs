// Mainly for the record
pub type PrvKey = [u8; 32];
pub type PubKey = [u8; 32];
pub type Nonce = [u8; 16];
pub type Sig = [u8; 64];

pub struct Witness {
    pub iou: u64,
    pub nonce: Nonce,
    pub sig: Sig,
}

impl Witness {
    pub fn from_proto(iou: &u64, nonce: &Vec<u8>, sig: &Vec<u8>) -> Self {
        let iou = iou.clone();
        let nonce = nonce.clone().try_into().unwrap();
        let sig = sig.clone().try_into().unwrap();
        Self { iou, nonce, sig }
    }
}

pub fn mk_message(iou: u64, nonce: &Nonce) -> Vec<u8> {
    nonce
        .clone()
        .into_iter()
        .chain(iou.to_be_bytes().into_iter())
        .collect::<Vec<u8>>()
}
