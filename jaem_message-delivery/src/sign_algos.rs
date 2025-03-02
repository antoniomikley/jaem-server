use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};

#[derive(FromRepr, EnumIter, Debug)]
#[repr(u8)]
pub enum AlgoSign {
    ED25519 = 0,
}

impl AlgoSign {
    pub fn get_key_len(&self) -> usize {
        match self {
            Self::ED25519 => return 32,
        }
    }

    pub fn get_signature_len(&self) -> usize {
        match self {
            Self::ED25519 => return 64,
        }
    }

    pub fn list() -> String {
        let mut list = String::from("");
        for algo in AlgoSign::iter() {
            list.push_str(&format!("{:?}", algo));
            list.push_str(", ");
        }
        return list[0..list.len() - 2].to_string();
    }
}
