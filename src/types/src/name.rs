//TODO I'm thinking we maybe make this a primitive. Not entirely sure yet, but I think a good
//discussion should be had on whether Name is a primitive or Type.
//Type to wrap Handshake names for type checkking as well as helper functions.
//TODO probably have to reimpl partialEq and Eq to just check strings.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Name(String);
