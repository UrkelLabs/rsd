//Type to wrap Handshake names for type checkking as well as helper functions.
//TODO probably have to reimpl partialEq and Eq to just check strings.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Name(String);
