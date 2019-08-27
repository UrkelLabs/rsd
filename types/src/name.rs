use std::ops::Deref;
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

impl From<String> for Name {
    fn from(name: String) -> Self {
        Name(name)
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//TODO names should not be valid beyond 63 characters. We need to replace the From<String> with a
//real FromStr trait so that it can fail on the various rules for names.
