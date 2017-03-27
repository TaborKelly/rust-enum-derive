extern crate getopts;
extern crate toml;
use std::env;
use getopts::Options;
use std::path::PathBuf;

extern crate rust_enum_derive;
use rust_enum_derive::*;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate regex;

#[derive(Debug, Default)]
struct Args {
    input: Option<String>,
    input_dir: Option<String>,
    output: Option<String>,
    output_dir: Option<String>,
}

fn parse_options() -> (Args, FileArgs) {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut a = Args::default();
    let mut fa = FileArgs::default();

    let mut opts = Options::new();
    opts.optopt("i", "input", "input file name (stdin if not specified)", "NAME");
    opts.optopt("", "input_dir", "input directory to traverse", "NAME");
    opts.optopt("o", "output", "output directory to traverse", "NAME");
    opts.optopt("", "output_dir", "output file name (stdout if not specified)", "NAME");
    opts.optopt("", "name", "the enum name (Name if not specified)", "NAME");
    opts.optopt("", "derive", "Which traits to derive. Ex: \"Debug, PartialEq\"", "DERIVE");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "define", "parse C #define input instead of enum");
    opts.optflag("a", "all", "implement all of the traits (equivalent to \
                 --display --fromprimative --fromstr)");
    opts.optflag("", "default", "implement the Default trait with the first \
                 value");
    opts.optflag("", "display", "implement the std::fmt::Display trait");
    opts.optflag("", "fromprimative", "use the enum_primitive crate to get from_primative trait");
    opts.optflag("", "fromstr", "implement the std::str::FromStr trait");
    opts.optflag("", "hex", "hexadecimal output");
    opts.optflag("", "pretty_fmt", "implement pretty_fmt()");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    a.input = matches.opt_str("i");
    a.input_dir = matches.opt_str("input_dir");
    a.output = matches.opt_str("o");
    a.output_dir = matches.opt_str("output_dir");
    fa.name = matches.opt_str("name");
    fa.derive = matches.opt_str("derive");
    fa.define = matches.opt_present("define");
    fa.default = matches.opt_present("default");
    fa.display = matches.opt_present("display");
    fa.fromprimative = matches.opt_present("fromprimative");
    fa.pretty_fmt = matches.opt_present("pretty_fmt");
    if fa.pretty_fmt {
        fa.fromprimative = true;
        fa.display = true;
    }
    fa.fromstr = matches.opt_present("fromstr");
    fa.hex = matches.opt_present("hex");
    if matches.opt_present("all") {
        fa.default = true;
        fa.display = true;
        fa.fromprimative = true;
        fa.fromstr = true;
        fa.pretty_fmt = true;
    }

    if a.input.is_some() && a.input_dir.is_some() {
        error!("using --input and --input_dir at the same time doesn't make \
               sense!");
        std::process::exit(1);
    }
    if a.output.is_some() && a.output_dir.is_some() {
        error!("using --output and --output_dir at the same time  doesn't \
               make sense!");
        std::process::exit(1);
    }
    if a.input_dir.is_some() && a.output_dir.is_none() {
        error!("if you use --input_dir you must use --output_dir!");
        std::process::exit(1);
    }

    (a, fa)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <options>\n\
                        A simple program for generating rust enums and \
                        associated traits from text files.",
                        program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    env_logger::init().unwrap();
    let (args, file_args) = parse_options();
    debug!("args = {:?}", args);

    if args.input_dir.is_some() {
        let input_dir = PathBuf::from(args.input_dir.as_ref().unwrap());
        let output_dir = PathBuf::from(args.output_dir.as_ref().unwrap());
        match traverse_dir(&input_dir, &output_dir)
        {
            Err(e) => panic!("{}", e),
            _ => ()
        }
    }
    else {
        // TODO: revisit. Is there a better way to do this?
        let file_path_in = match args.input {
            Some(ref s) => Some(PathBuf::from(s)),
            None => None,
        };
        let file_path_in_ref = match file_path_in {
            Some(ref fp) => Some(fp),
            None => None,
        };

        // TODO: revisit. Is there a better way to do this?
        let file_path_out = match args.output {
            Some(ref s) => Some(PathBuf::from(s)),
            None => None,
        };
        let file_path_out_ref = match file_path_out {
            Some(ref fp) => Some(fp),
            None => None,
        };

        match process(file_path_in_ref, file_path_out_ref, &file_args)
        {
            Err(e) => panic!("{}", e),
            _ => ()
        }
    }
}
