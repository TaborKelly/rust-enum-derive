extern crate getopts;
extern crate toml;
use std::env;
use getopts::Options;
use std::cmp::Ordering;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result};
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;

#[macro_use]
extern crate log;
extern crate env_logger; // TODO: replace
extern crate regex;

// TODO: add more tests

#[derive(Debug, Default)]
struct Args {
    input: Option<String>,
    input_dir: Option<String>,
    output: Option<String>,
    output_dir: Option<String>,
}
#[derive(Debug)]
struct FileArgs {
    name: Option<String>,
    derive: Option<String>,
    define: bool,
    default: bool,
    display: bool,
    fromstr: bool,
    fromprimative: bool,
    hex: bool,
    pretty_fmt: bool,
}
impl Default for FileArgs {
    fn default() -> FileArgs
    {
        FileArgs{ name: None, derive: None, define: false, default: false, display: false,
                 fromstr: false, fromprimative: false, hex: false, pretty_fmt: false }
    }
}
trait FormatOutput {
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> Result<()>;
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
    opts.optflag("", "fromprimative", "implement the num::traits::FromPrimitive trait");
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

fn get_num(s: &str) -> i32 {
    use std::str::FromStr;
    use regex::Regex;
    let re_int = Regex::new(r"^(0x)?([:digit:]+)$").unwrap();
    let re_shift = Regex::new(r"^([:digit:]+)[:space:]*<<[:space:]*([:digit:]+)$").unwrap();

    if re_int.is_match(s) {
        let caps = re_int.captures(s).unwrap();
        let radix: u32 = match caps.at(1) {
            Some(_) => 16,
            None => 10,
        };
        let digits = caps.at(2).unwrap();
        i32::from_str_radix(digits, radix).unwrap()
    }
    else if re_shift.is_match(s) {
        let caps = re_shift.captures(s).unwrap();
        let l: i32 = FromStr::from_str(caps.at(1).unwrap()).unwrap();
        let r: i32 = FromStr::from_str(caps.at(2).unwrap()).unwrap();
        l<<r
    }
    else {
        panic!("couldn't parse '{}' as int", s)
    }
}

/// Return a sorted Vec of CEnum structs
fn parse_buff<T: BufRead>(read: T, parse_enum: bool) -> Vec<CEnum> {
    use regex::Regex;
    let re = match parse_enum {
        true => Regex::new(r"^[:space:]*([[:alnum:]_]+)([:space:]*=[:space:]*([:graph:]+))?[:space:]*,").unwrap(),
        false => Regex::new(r"^#define[:space:]+([:graph:]+)[:space:]+([:graph:]+)").unwrap(),
    };
    let mut v: Vec<CEnum> = Vec::new();

    let mut num: i32 = 0;
    for line in read.lines() {
        let s = line.unwrap();
        for cap in re.captures_iter(&s) {
            let i: i32 = match parse_enum {
                true => match cap.at(3) {
                    Some(s) => get_num(s),
                    None => num,
                },
                false => get_num(cap.at(2).unwrap()),
            };
            num = i + 1;
            v.push(CEnum::new(i, cap.at(1).unwrap()));
        }
    }

    v.sort();
    v
}

fn get_input(file_path: Option<&PathBuf>, file_args: &FileArgs) -> Vec<CEnum> {
    match file_path {
        Some(ref s) => {
            // remove this unwrap as soon as expect is stabalized
            let f = File::open(s).unwrap();
            let r = BufReader::new(f);
            parse_buff(r, !file_args.define)
        }
        None => {
            let r = BufReader::new(std::io::stdin());
            parse_buff(r, !file_args.define)
        }
    }
}

struct FormatOutputFromPrimative;
impl FormatOutput for FormatOutputFromPrimative {
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> Result<()> {
        try!(write!(w, "impl ::num::traits::FromPrimitive for {} {{\n", name));
        try!(write!(w, "    #[allow(dead_code)]\n"));
        try!(write!(w, "    fn from_i64(n: i64) -> Option<Self> {{\n"));
        try!(write!(w, "        match n {{\n"));
        for v in vec {
            if hex {
                try!(write!(w, "            0x{:X} => Some({}::{}),\n", v.i, name, v.s));
            }
            else {
                try!(write!(w, "            {} => Some({}::{}),\n", v.i, name, v.s));
            }
        }
        try!(write!(w, "            _ => None\n"));
        try!(write!(w, "        }}\n"));
        try!(write!(w, "    }}\n"));
        try!(write!(w, "    #[allow(dead_code)]\n"));
        try!(write!(w, "    fn from_u64(n: u64) -> Option<Self> {{\n"));
        try!(write!(w, "        match n {{\n"));
        for v in vec {
            if hex {
                try!(write!(w, "            0x{:X} => Some({}::{}),\n", v.i, name, v.s));
            }
            else {
                try!(write!(w, "            {} => Some({}::{}),\n", v.i, name, v.s));
            }
        }
        try!(write!(w, "            _ => None\n"));
        try!(write!(w, "        }}\n"));
        try!(write!(w, "    }}\n"));
        try!(write!(w, "}}\n"));
        Ok(())
    }
}

struct FormatOutputPrettyFmt;
impl FormatOutput for FormatOutputPrettyFmt {
    #[allow(unused_variables)]
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> Result<()> {
        try!(write!(w, "impl {} {{\n", name));
        try!(write!(w, "    fn pretty_fmt(f: &mut ::std::fmt::Formatter, flags: u32) -> ::std::fmt::Result {{\n"));
        try!(write!(w, "        let mut shift: u32 = 0;\n"));
        try!(write!(w, "        let mut result: u32 = 1<<shift;\n"));
        try!(write!(w, "        let mut found = false;\n"));
        // This should never fail because we check in main() to make sure that
        // it isn't empty.
        try!(write!(w, "        while result <= {}::{} as u32 {{\n", name, vec.last().unwrap().s));
        try!(write!(w, "            let tmp = result & flags;\n"));
        try!(write!(w, "            if tmp > 0 {{\n"));
        try!(write!(w, "                if found {{\n"));
        try!(write!(w, "                    try!(write!(f, \"|\"));\n"));
        try!(write!(w, "                }}\n"));
        try!(write!(w, "                let flag = {}::from_u32(tmp).unwrap();\n", name));
        try!(write!(w, "                try!(write!(f, \"{{}}\", flag));\n"));
        try!(write!(w, "                found = true;\n"));
        try!(write!(w, "            }}\n"));
        try!(write!(w, "            shift += 1;\n"));
        try!(write!(w, "            result = 1<<shift;\n"));
        try!(write!(w, "        }}\n"));
        try!(write!(w, "        write!(f, \"\")\n"));
        try!(write!(w, "    }}\n"));
        try!(write!(w, "}}\n"));
        Ok(())
    }
}

struct FormatOutputDefault;
impl FormatOutput for FormatOutputDefault {
    #[allow(unused_variables)]
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> Result<()> {
        try!(write!(w, "impl Default for {} {{\n", name));
        try!(write!(w, "    fn default() -> {} {{\n", name));
        try!(write!(w, "        {}::{}\n", name, vec[0].s));
        try!(write!(w, "    }}\n"));
        try!(write!(w, "}}\n"));
        Ok(())
    }
}

struct FormatOutputDisplay;
impl FormatOutput for FormatOutputDisplay {
    #[allow(unused_variables)]
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> Result<()> {
        try!(write!(w, "impl ::std::fmt::Display for {} {{\n", name));
        try!(write!(w, "    #[allow(dead_code)]\n"));
        try!(write!(w, "    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{\n"));
        try!(write!(w, "        match *self {{\n"));
        for v in vec {
            try!(write!(w, "            {}::{} => write!(f, \"{}\"),\n", name, v.s, v.s));
        }
        try!(write!(w, "        }}\n"));
        try!(write!(w, "    }}\n"));
        try!(write!(w, "}}\n"));
        Ok(())
    }
}

struct FormatOutputFromStr;
impl FormatOutput for FormatOutputFromStr {
    #[allow(unused_variables)]
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> Result<()> {
        try!(write!(w, "impl ::std::str::FromStr for {} {{\n", name));
        try!(write!(w, "    type Err = ();\n"));
        try!(write!(w, "    #[allow(dead_code)]\n"));
        try!(write!(w, "    fn from_str(s: &str) -> Result<Self, Self::Err> {{\n"));
        try!(write!(w, "        match s {{\n"));
        for v in vec {
            try!(write!(w, "            \"{}\" => Ok({}::{}),\n", v.s, name, v.s));
        }
        try!(write!(w, "            _ => Err( () )\n"));
        try!(write!(w, "        }}\n"));
        try!(write!(w, "    }}\n"));
        try!(write!(w, "}}\n"));
        Ok(())
    }
}

struct FormatOutputEnum;
impl FormatOutputEnum {
    fn write(&self, w: &mut Write, name: &String, derive: Option<&String>, hex: bool, vec: &Vec<CEnum>) -> Result<()> {
        try!(write!(w, "#[allow(dead_code, non_camel_case_types)]\n"));
        match derive
        {
            Some(s) => try!(write!(w, "#[derive({})]\n", s)),
            None => (),
        }
        try!(write!(w, "pub enum {} {{\n", name));

        for v in vec {
            if hex {
                try!(write!(w, "    {} = 0x{:X},\n", v.s, v.i));
            }
            else {
                try!(write!(w, "    {} = {},\n", v.s, v.i));
            }
        }

        try!(write!(w, "}}\n"));
        Ok(())
    }
}

fn write_factory(file_path: Option<&PathBuf>) -> Result<Box<Write>> {
    match file_path {
        Some(s) => {
            try!(std::fs::create_dir_all(s.parent().unwrap()));
            let f = try!(OpenOptions::new().write(true)
                                           .create(true)
                                           .truncate(true)
                                           .open(s));
            let w = BufWriter::new(f);
            Ok(Box::new(w))
        }
        None => {
            let w = BufWriter::new(std::io::stdout());
            Ok(Box::new(w))
        }
    }
}

// A macro to retrieve an str element from a toml::Table
// $t - Table to lookup in
// $a - Where to assign Some(String)
// $v - the name to look for in the toml
macro_rules! get_key_string {
    ($t:ident, $a:ident, $v:ident) => {
        if $t.contains_key(stringify!($v)) {
            let $v = $t.get(stringify!($v)).unwrap();
            let $v = $v.as_str();
            if $v.is_none() {
                return Err(Error::new(ErrorKind::Other,
                                      format!("{} wasn't available as str",
                                              stringify!($v))))
            }
            let $v = $v.unwrap();
            $a.$v = Some(String::from($v));
        }
    }
}

// same as get_key_bool, except for bool instead of str/string
macro_rules! get_key_bool {
    ($t:ident, $a:ident, $v:ident) => {
        if $t.contains_key(stringify!($v)) {
            let $v = $t.get(stringify!($v)).unwrap();
            let $v = $v.as_bool();
            if $v.is_none() {
                return Err(Error::new(ErrorKind::Other,
                                      format!("{} wasn't available as bool",
                                              stringify!($v))))
            }
            $a.$v = $v.unwrap();
        }
    }
}

fn parse_toml(path: &PathBuf) -> Result<FileArgs>
{
    let mut fa = FileArgs::default();
    let mut f = try!(File::open(&path));

    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    let table = toml::Parser::new(&s).parse();
    if table.is_none() {
        return Err(Error::new(ErrorKind::Other,
                              format!("failed to parse {}", path.display())))
    }
    let table = table.unwrap();

    let rust_enum_derive = table.get("rust-enum-derive");
    if rust_enum_derive.is_none() {
        return Err(Error::new(ErrorKind::Other,
                              format!("couldn't find a rust-enum-derive table in {}",
                                      path.display())))
    }
    let rust_enum_derive = rust_enum_derive.unwrap();
    let rust_enum_derive = rust_enum_derive.as_table();
    if rust_enum_derive.is_none() {
        return Err(Error::new(ErrorKind::Other,
                              format!("rust-enum-derive wasn't a table")))
    }
    let rust_enum_derive = rust_enum_derive.unwrap();

    get_key_string!(rust_enum_derive, fa, name);
    get_key_string!(rust_enum_derive, fa, derive);
    get_key_bool!(rust_enum_derive, fa, define);
    get_key_bool!(rust_enum_derive, fa, default);
    get_key_bool!(rust_enum_derive, fa, display);
    get_key_bool!(rust_enum_derive, fa, fromstr);
    get_key_bool!(rust_enum_derive, fa, fromprimative);
    get_key_bool!(rust_enum_derive, fa, hex);
    get_key_bool!(rust_enum_derive, fa, pretty_fmt);
    debug!("fa = {:?}", fa);

    Ok(fa)
}

fn process(file_path_in: Option<&PathBuf>, file_path_out: Option<&PathBuf>,
           file_args: &FileArgs) -> Result<()> {
    let mut fov: Vec<Box<FormatOutput>> = Vec::new();
    if file_args.fromstr { fov.push(Box::new(FormatOutputFromStr)); }
    if file_args.default { fov.push(Box::new(FormatOutputDefault)); }
    if file_args.display { fov.push(Box::new(FormatOutputDisplay)); }
    if file_args.fromprimative { fov.push(Box::new(FormatOutputFromPrimative)); }
    if file_args.pretty_fmt { fov.push(Box::new(FormatOutputPrettyFmt)); }

    let vi = get_input(file_path_in, &file_args);
    if vi.len() < 1 {
        let input = match file_path_in {
            Some(pb) => pb.to_string_lossy().into_owned(),
            None => String::from("standard in"),
        };
        return Err(Error::new(ErrorKind::Other,
                              format!("couldn't parse any input from {}.",
                                      input)))
    }

    let mut w = try!(write_factory(file_path_out));
    let name = match file_args.name
    {
        Some(ref s) => s.clone(),
        None => String::from("Name"),
    };

    let derive = file_args.derive.as_ref();
    try!(FormatOutputEnum.write(&mut w, &name, derive, file_args.hex, &vi));
    for vw in fov {
        try!(vw.write(&mut w, &name, file_args.hex, &vi));
    }

    Ok(())
}

fn traverse_dir(base_input_dir: &PathBuf,
                base_output_dir: &PathBuf,
                sub_dir: &PathBuf) -> Result<()>{
    let mut dir = PathBuf::new();
    dir.push(base_input_dir);
    dir.push(sub_dir);

    if !fs::metadata(&dir).unwrap().is_dir() {
        return Err(Error::new(ErrorKind::Other,
                              format!("{} is not a directory", dir.display())))
    }

    // TODO: revisit. This follows symlinks, is that what we want?
    // If no we could use fs::symlink_metadata() treats symbolic links as
    // files, or DirEntry::file_type() which returns a FileType which we could
    // use to tell if this was a symbolic link or not?
    for entry in try!(fs::read_dir(dir)) {
        let entry = try!(entry);
        if fs::metadata(entry.path()).unwrap().is_dir() {
            let mut new_sub_dir = PathBuf::new();
            new_sub_dir.push(sub_dir);
            new_sub_dir.push(entry.file_name());
            try!(traverse_dir(base_input_dir, base_output_dir, &new_sub_dir));
        } else {
            let path = entry.path();
            if path.extension().is_some() {
                let extension = path.extension().unwrap();
                let extension = extension.to_string_lossy();
                let extension = extension.to_lowercase();
                if extension == "toml" {
                    let args = try!(parse_toml(&path));

                    let path = entry.path();
                    let base = path.file_stem().unwrap();

                    let mut input_file_path = PathBuf::new();
                    input_file_path.push(base_input_dir);
                    input_file_path.push(sub_dir);
                    input_file_path.push(base);
                    input_file_path.set_extension("in");

                    let mut output_file_path = PathBuf::new();
                    output_file_path.push(base_output_dir);
                    output_file_path.push(sub_dir);
                    output_file_path.push(base);
                    output_file_path.set_extension("rs");

                    try!(process(Some(&input_file_path), Some(&output_file_path), &args));
                }
            }
        }
    } // for entry in try!(fs::read_dir(dir))

    Ok(())
}

fn main() {
    env_logger::init().unwrap();
    let (args, file_args) = parse_options();
    debug!("args = {:?}", args);

    if args.input_dir.is_some() {
        let input_dir = PathBuf::from(args.input_dir.as_ref().unwrap());
        let output_dir = PathBuf::from(args.output_dir.as_ref().unwrap());
        match traverse_dir(&input_dir, &output_dir, &PathBuf::new())
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

#[derive(Debug)]
struct CEnum {
    i: i32,
    s: String,
}
impl CEnum {
    fn new(i: i32, s: &str) -> CEnum {
        CEnum { i:i, s: String::from(s) }
    }
}
impl ::std::cmp::Eq for CEnum {}
impl ::std::cmp::PartialEq for CEnum {
    fn eq(&self, other: &Self) -> bool {
        if self.i == other.i {
            return true;
        }
        false
    }
}
impl ::std::cmp::PartialOrd for CEnum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.i < other.i {
            return Some(Ordering::Less);
        }
        else if self.i > other.i {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}
impl ::std::cmp::Ord for CEnum {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.i < other.i {
            return Ordering::Less;
        }
        else if self.i > other.i {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

#[test]
fn test_CENum_order() {
    let a = CEnum::new(0, "");
    let b = CEnum::new(1, "");
    let c = CEnum::new(2, "");
    let d = CEnum::new(0, "");
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
    assert!(b > a);
    assert!(c > b);
    assert!(c > a);
    assert!(a == d);
}

#[test]
fn test_parse_buff() {
    use std::io::Cursor;
    let s = "#define NETLINK_ROUTE 0\n\
    #define NETLINK_UNUSED 1\n\
    #define NETLINK_FIREWALL 3\n\
    #define NETLINK_SOCK_DIAG 4\n\
    #define NETLINK_GENERIC 16";

    let buff = Cursor::new(s.as_bytes());

    let v = parse_buff(buff, false);

    assert!(v[0].i == 0); assert!(v[0].s == "NETLINK_ROUTE");
    assert!(v[1].i == 1); assert!(v[1].s == "NETLINK_UNUSED");
    assert!(v[2].i == 3); assert!(v[2].s == "NETLINK_FIREWALL");
    assert!(v[3].i == 4); assert!(v[3].s == "NETLINK_SOCK_DIAG");
    assert!(v[4].i == 16); assert!(v[4].s == "NETLINK_GENERIC");
}

#[test]
fn test_parse_buff_enum() {
    use std::io::Cursor;
    let s = "RTM_NEWLINK    = 16,\n\
             #define RTM_NEWLINK    RTM_NEWLINK\n\
                 RTM_DELLINK,\n\
             #define RTM_DELLINK    RTM_DELLINK\n\
                 RTM_GETLINK,\n\
             #define RTM_GETLINK    RTM_GETLINK\n\
                 RTM_SETLINK,\n\
             #define RTM_SETLINK    RTM_SETLINK\n\n\
                 RTM_NEWADDR    = 20,\n\
             #define RTM_NEWADDR    RTM_NEWADDR\n\
                 RTM_DELADDR,";

    let buff = Cursor::new(s.as_bytes());
    let v = parse_buff(buff, true);

    assert!(v[0].i == 16); assert!(v[0].s == "RTM_NEWLINK");
    assert!(v[1].i == 17); assert!(v[1].s == "RTM_DELLINK");
    assert!(v[2].i == 18); assert!(v[2].s == "RTM_GETLINK");
    assert!(v[3].i == 19); assert!(v[3].s == "RTM_SETLINK");
    assert!(v[4].i == 20); assert!(v[4].s == "RTM_NEWADDR");
    assert!(v[5].i == 21); assert!(v[5].s == "RTM_DELADDR");
}
