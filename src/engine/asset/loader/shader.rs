use engine::asset::loader::Loader;
use engine::asset::{AssetError, File};
use engine::render::{ShaderFs, ShaderVs};

use std::str;

pub struct ShaderVSLoader {}
pub struct ShaderFSLoader {}

impl Loader<ShaderVs> for ShaderVSLoader {
    fn load(mut file: Box<File>) -> Result<ShaderVs, AssetError> {
        let buf = file.read_binary().map_err(|_| AssetError::InvalidFormat)?;
        let vs = str::from_utf8(&buf).map_err(|_| AssetError::InvalidFormat)?;
        Ok(ShaderVs::new(&file.name(), vs))
    }
}

impl Loader<ShaderFs> for ShaderFSLoader {
    fn load(mut file: Box<File>) -> Result<ShaderFs, AssetError> {
        let buf = file.read_binary().map_err(|_| AssetError::InvalidFormat)?;
        let fs = str::from_utf8(&buf).map_err(|_| AssetError::InvalidFormat)?;

        Ok(ShaderFs::new(&file.name(), fs))
    }
}
