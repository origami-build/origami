pub mod javacli;
pub mod protocol;
pub mod framed;

pub mod jvm;

#[cfg(not(any(target_os = "windows", target_os = "redox")))]
const PATH_SEPARATOR: &str = ":";

#[cfg(any(target_os = "windows", target_os = "redox"))]
const PATH_SEPARATOR: &str = ";";
