use std::path::Path;

use clap::{App, Arg, ArgGroup, ArgMatches};

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
                .multiple_occurrences(true)
                .help("Include source file paths"),
            Arg::new("link")
                .short('l')
                .long("link")
                .value_name("path")
                .multiple_occurrences(true)
                .help("Link against compiled JAR/class file paths"),
            Arg::new("out-dir")
                .short('o')
                .long("out-dir")
                .value_name("dir")
                .help("Specify output directory (default: package root)"),
            Arg::new("package-root")
                .long("package-root")
                .value_name("dir")
                .help("Specify package root (default: auto-detected)"),
            Arg::new("debug")
                .short('g')
                .long("debug")
                .help("Generate debugging information"),
            Arg::new("release")
                .long("release")
                .value_name("version")
                .help("Set Java SE release to compile against"),
            Arg::new("in-file")
                .value_name("source-file")
                .required(true)
                .multiple_occurrences(true)
                .help("Java source file"),
            Arg::new("write-deps")
                .long("write-deps")
                .value_name("file")
                .help("Write dependency manifest"),
            Arg::new("write-makedeps")
                .long("write-makedeps")
                .value_name("file")
                .help("Write dependency manifest in Make format"),
            Arg::new("ap-args")
                .short('A')
                .value_name("definition")
                .multiple_occurrences(true)
                .help("Pass options to annotation processors"),
            Arg::new("no-class-gen")
                .short('E')
                .help("Don't output class files"),
            Arg::new("no-ap")
                .short('P')
                .help("Don't run annotation processors"),
        ])
        .groups([ArgGroup::new("output-config").args(&["no-class-gen", "no-ap"])])
    }
}

pub fn read_props(matches: &ArgMatches) -> CommonProps {
    let in_files = matches
        .values_of("in-file")
        .map(|iter| iter.map(Path::new).collect())
        .unwrap_or_default();
    let include = matches
        .values_of("include")
        .map(|iter| iter.map(Path::new).collect())
        .unwrap_or_default();
    let link = matches
        .values_of("link")
        .map(|iter| iter.map(Path::new).collect())
        .unwrap_or_default();
    let out_dir = matches.value_of("out-dir").map(Path::new);
    let package_root = matches.value_of("package-root").map(Path::new);
    let debug = matches.is_present("debug");
    let release = matches.value_of("release");
    let write_deps = matches.value_of("write-deps").map(Path::new);
    let write_makedeps = matches.value_of("write-makedeps").map(Path::new);
    let ap_args = matches.values_of("ap-args").into_iter().flatten().collect();
    let no_ap = matches.is_present("no-ap");
    let no_class_gen = matches.is_present("no-class-gen");

    CommonProps {
        in_files,
        include,
        link,
        out_dir,
        package_root,
        debug,
        release,
        write_deps,
        write_makedeps,
        ap_args,
        no_ap,
        no_class_gen,
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
    pub write_deps: Option<&'a Path>,

    /// The path to output a manifest of input and output files for. If None,
    /// does not write the manifest.
    pub write_makedeps: Option<&'a Path>,

    pub ap_args: Vec<&'a str>,
    pub no_ap: bool,
    pub no_class_gen: bool,
}
