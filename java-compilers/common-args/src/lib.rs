use std::path::Path;

use clap::{App, Arg, ArgMatches};

pub trait AppExt {
    fn add_javac_common_args(self) -> Self;
}

impl<'a> AppExt for App<'a> {
    fn add_javac_common_args(self) -> Self {
        self.args(&[
            Arg::new("include")
                .short('I')
                .long("include")
                .value_name("path")
                .multiple(true)
                .about("Include source file paths"),
            Arg::new("link")
                .short('l')
                .long("link")
                .value_name("path")
                .multiple(true)
                .about("Link against compiled JAR/class file paths"),
            Arg::new("out-dir")
                .short('o')
                .long("out-dir")
                .value_name("dir")
                .about("Specify output directory (default: package root)"),
            Arg::new("package-root")
                .long("package-root")
                .value_name("dir")
                .about("Specify package root (default: auto-detected)"),
            Arg::new("debug")
                .short('g')
                .long("debug")
                .about("Generate debugging information"),
            Arg::new("release")
                .long("release")
                .value_name("version")
                .about("Set Java SE release to compile against"),
            Arg::new("in-file")
                .value_name("source-file")
                .required(true)
                .multiple(true)
                .about("Java source file"),
            Arg::new("manifest")
                .long("manifest")
                .value_name("file")
                .about("Write dependency manifest"),
        ])
    }
}

pub fn read_props(matches: &ArgMatches) -> CommonProps {
    let in_files = matches
        .values_of_os("in-file")
        .map(|iter| iter.map(Path::new).collect())
        .unwrap_or_default();
    let include = matches
        .values_of_os("include")
        .map(|iter| iter.map(Path::new).collect())
        .unwrap_or_default();
    let link = matches
        .values_of_os("link")
        .map(|iter| iter.map(Path::new).collect())
        .unwrap_or_default();
    let out_dir = matches.value_of_os("out-dir").map(Path::new);
    let package_root = matches.value_of_os("package-root").map(Path::new);
    let debug = matches.is_present("debug");
    let release = matches.value_of("release");
    let manifest = matches.value_of_os("manifest").map(Path::new);

    CommonProps {
        in_files,
        include,
        link,
        out_dir,
        package_root,
        debug,
        release,
        manifest,
    }
}

pub struct CommonProps<'a> {
    /// The source files to be compiled.
    pub in_files: Vec<&'a Path>,

    /// The directories the compiler should look for source files referred to by
    /// the input files in.
    pub include: Vec<&'a Path>,

    /// The directories or JAR files the compiler should look for class files
    /// referred to by the input files in.
    pub link: Vec<&'a Path>,

    /// The directory to place compiled files in.
    pub out_dir: Option<&'a Path>,

    pub package_root: Option<&'a Path>,

    /// Whether to include debug information in the compiled files.
    pub debug: bool,

    /// The JVM release version to compile for.
    pub release: Option<&'a str>,
    
    /// The path to output a manifest of input and output files for. If None,
    /// does not write the manifest.
    pub manifest: Option<&'a Path>,
}
