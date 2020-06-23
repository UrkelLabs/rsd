use std::ops::Deref;
use std::str::FromStr;

static MAX_NAME_SIZE: usize = 63;

static BLACKLIST: &[&str] = &["example", "invalid", "local", "localhost", "test"];

#[derive(Debug)]
pub enum NameError {
    InvalidName,
}

//TODO I'm thinking we maybe make this a primitive. Not entirely sure yet, but I think a good
//discussion should be had on whether Name is a primitive or Type.
//Type to wrap Handshake names for type checkking as well as helper functions.
//TODO probably have to reimpl partialEq and Eq to just check strings.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Name(String);

impl Name {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Name {
    type Err = NameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            //@todo could go the more descriptive route on this and say invalidLength.
            return Err(NameError::InvalidName);
        }

        if s.len() > MAX_NAME_SIZE {
            return Err(NameError::InvalidName);
        }

        if !s.is_ascii() {
            return Err(NameError::InvalidName);
        }

        //No capital
        if s.contains(|c: char| c.is_uppercase()) {
            return Err(NameError::InvalidName);
        }

        if s.starts_with('_') || s.ends_with('_') {
            return Err(NameError::InvalidName);
        }

        if s.starts_with('-') || s.ends_with('-') {
            return Err(NameError::InvalidName);
        }

        //Check blacklist.
        if BLACKLIST.contains(&s) {
            return Err(NameError::InvalidName);
        }

        Ok(Name(s.to_string()))
    }
}
