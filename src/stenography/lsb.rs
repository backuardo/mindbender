use crate::error::ApplicationError;

pub struct LsbCodec;

impl LsbCodec {
    pub fn encode(data_path: String, carrier_path: String) -> Result<(), ApplicationError> {
        todo!()
    }

    pub fn decode(path: String) -> Result<(), ApplicationError> {
        todo!()
    }
}
