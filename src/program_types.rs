
use failure;

use error::{WebGuiError};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProgramType {
    PecubeESD,
    Grain3DHe,
    LandLabESD,
    IceCascade,
    CoupledLandscapeThermalSimulator,
}

impl ProgramType {
    pub fn convert(num: u8) -> Result<ProgramType, failure::Error> {
        use self::ProgramType::*;

        match num {
            0 => Ok(PecubeESD),
            1 => Ok(Grain3DHe),
            2 => Ok(LandLabESD),
            3 => Ok(IceCascade),
            4 => Ok(CoupledLandscapeThermalSimulator),
            _ => Err(WebGuiError::UnknownProgramType.into()),
        }
    }
}
