use bypar::prelude::{SizedString, SizedVec};
use bypar_derive::{FromBytes, ToBytes};

#[derive(ToBytes, FromBytes)]
pub enum Communication {
    #[enum_index(0)]
    Inputs(SizedVec<u32, SizedString<u32>>),
    #[enum_index(1)]
    Output(SizedString<u32>),
}
