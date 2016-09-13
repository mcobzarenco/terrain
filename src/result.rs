// use std::fmt;
// use std::error::Error;

// #[derive(Debug)]
// pub enum TerrainError {
//     AssetError(String),
// }

// impl fmt::Display for TerrainError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             TerrainError::AssetError(ref msg) => f.write_str(&msg),
//         }
//     }
// }

// impl Error for TerrainError {
//     fn description(&self) -> &str {
//         match *self {
//             TerrainError::AssetError(ref msg) => msg.as_str(),
//         }
//     }

//     fn cause(&self) -> Option<&Error> {
//         match *self {
//             TerrainError::AssetError(_) => None,
//         }
//     }
// }

// pub type TerrainResult<V> = Result<V, TerrainError>;
