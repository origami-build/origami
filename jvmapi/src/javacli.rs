use std::ffi::OsString;
use std::path::Path;

use crate::PATH_SEPARATOR;

pub fn build_classpath<I>(classpath: I) -> Option<OsString>
where
    I: IntoIterator,
    I::Item: AsRef<Path>,
{
    let mut classpath = classpath.into_iter();

    classpath.next().map(|first| {
        classpath.fold(
            first.as_ref().to_path_buf().into_os_string(),
            |mut acc, el| {
                acc.push(PATH_SEPARATOR);
                acc.push(el.as_ref());
                acc
            },
        )
    })
}

pub fn jvm_args<I>(classpath: I, main_class: &str) -> Vec<OsString>
where
    I: IntoIterator,
    I::Item: AsRef<Path>,
{
    let classpath = build_classpath(classpath);
    let mut out = vec![];

    if let Some(cp_str) = classpath {
        out.push("-classpath".into());
        out.push(cp_str);
    }

    out.push(main_class.into());

    out
}
