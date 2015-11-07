extern crate getopts;
use std::env;
use getopts::Options;
use std::cmp::Ordering;
use std::io::{Write, BufRead, BufReader, BufWriter};
use std::fs::{File, OpenOptions};

#[macro_use]
extern crate log;
extern crate env_logger; // TODO: replace

extern crate regex;

// TODO: add more tests

#[derive(Debug)]
#[derive(Default)]
struct Args {
    input: Option<String>,
    output: Option<String>,
    name: String,
    c_enum: bool,
    default: bool,
    display: bool,
    fromstr: bool,
    fromprimative: bool,
    hex: bool,
    pretty_fmt: bool,
}

trait FormatOutput {
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()>;
}

fn parse_options() -> Args {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut a = Args::default();

    let mut opts = Options::new();
    opts.optopt("i", "input", "input file name (stdin if not specified)", "NAME");
    opts.optopt("o", "output", "output file name (stdout if not specified)", "NAME");
    opts.optopt("", "name", "the enum name (Name if not specified)", "NAME");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("", "enum", "parse C enum input instead of #define");
    opts.optflag("a", "all", "implment all of the traits (equivalent to \
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
    a.output = matches.opt_str("o");
    let name = matches.opt_str("name");
    // apply default name
    a.name = name.unwrap_or(String::from("Name"));
    a.c_enum = matches.opt_present("enum");
    a.default = matches.opt_present("default");
    a.display = matches.opt_present("display");
    a.fromprimative = matches.opt_present("fromprimative");
    a.pretty_fmt = matches.opt_present("pretty_fmt");
    if a.pretty_fmt {
        a.fromprimative = true;
        a.display = true;
    }
    a.fromstr = matches.opt_present("fromstr");
    a.hex = matches.opt_present("hex");
    if matches.opt_present("all") {
        a.default = true;
        a.display = true;
        a.fromprimative = true;
        a.fromstr = true;
        a.pretty_fmt = true;
    }

    a
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <options>\n\
                        Crudely converts C #defines into Rust enums.",
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

fn get_input(args: &Args) -> Vec<CEnum> {
    match args.input {
        Some(ref s) => {
            // remove this unwrap as soon as expect is stabalized
            let f = File::open(s).unwrap();
            let r = BufReader::new(f);
            parse_buff(r, args.c_enum)
        }
        None => {
            let r = BufReader::new(std::io::stdin());
            parse_buff(r, args.c_enum)
        }
    }
}

struct FormatOutputFromPrimative;
impl FormatOutput for FormatOutputFromPrimative {
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()> {
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
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()> {
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
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()> {
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
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()> {
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
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()> {
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
impl FormatOutput for FormatOutputEnum {
    fn write(&self, w: &mut Write, name: &String, hex: bool, vec: &Vec<CEnum>) -> ::std::io::Result<()> {
        try!(write!(w, "#[allow(dead_code, non_camel_case_types)]\n"));
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

fn write_factory(args: &Args) -> Box<Write> {
    match args.output {
        Some(ref s) => {
            let f = OpenOptions::new().write(true)
                                      .create(true)
                                      .truncate(true)
                                      // remove this unwrap as soon as expect
                                      // is stabalized
                                      .open(s).unwrap();
            let w = BufWriter::new(f);
            Box::new(w)
        }
        None => {
            let w = BufWriter::new(std::io::stdout());
            Box::new(w)
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = parse_options();
    debug!("args = {:?}", args);

    let mut fov: Vec<Box<FormatOutput>> = Vec::new();
    fov.push(Box::new(FormatOutputEnum));
    if args.fromstr { fov.push(Box::new(FormatOutputFromStr)); }
    if args.default { fov.push(Box::new(FormatOutputDefault)); }
    if args.display { fov.push(Box::new(FormatOutputDisplay)); }
    if args.fromprimative { fov.push(Box::new(FormatOutputFromPrimative)); }
    if args.pretty_fmt { fov.push(Box::new(FormatOutputPrettyFmt)); }

    let vi = get_input(&args);
    if vi.len() < 1 {
        println!("Error: couldn't parse any input. Try turning --enum off/on.");
        return;
    }
    let mut w = write_factory(&args);

    for vw in fov {
        match vw.write(&mut w, &args.name, args.hex, &vi)
        {
            Err(e) => println!("Error: {}", e),
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
