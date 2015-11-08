# rust-enum-derive

rust-enum-derive is a simple program for generating rust enums and associated
traits from text files. To simplify converting C code these text files are
allowed to look like C enums or C #defines.

## Usage
```
Usage: ./rust-enum-derive <options>
Crudely converts C enums (or #defines) into Rust enums.
Options:
    -i, --input NAME    input file name (stdin if not specified)
    -o, --output NAME   output file name (stdout if not specified)
        --name NAME     the enum name (Name if not specified)
    -h, --help          print this help menu
        --define        parse C #define input instead of enum
    -a, --all           implement all of the traits (equivalent to --display
                        --fromprimative --fromstr)
        --default       implement the Default trait with the first value
        --display       implement the std::fmt::Display trait
        --fromprimative
                        implement the num::traits::FromPrimitive trait
        --fromstr       implement the std::str::FromStr trait
        --hex           hexadecimal output
        --pretty_fmt    implement pretty_fmt()
```

## Simple examples
All of the following examples produce the same code:

```
ZERO = 0,
ONE = 1,
TWO = 2,
```

```
ZERO,
ONE,
TWO,
```

```
#define ZERO 0
#define ONE 1
#define TWO 2
```

This is the rust enum that will result:

```
pub enum Name {
    ZERO = 0,
    ONE = 1,
    TWO = 2,
}
```

## Complex example
For example, rust-enum-derive can take the following text file (linux/if.h):

```
enum net_device_flags {
	IFF_UP				= 1<<0,  /* sysfs */
	IFF_BROADCAST			= 1<<1,  /* __volatile__ */
	IFF_DEBUG			= 1<<2,  /* sysfs */
	IFF_LOOPBACK			= 1<<3,  /* __volatile__ */
	IFF_POINTOPOINT			= 1<<4,  /* __volatile__ */
	IFF_NOTRAILERS			= 1<<5,  /* sysfs */
	IFF_RUNNING			= 1<<6,  /* __volatile__ */
	IFF_NOARP			= 1<<7,  /* sysfs */
	IFF_PROMISC			= 1<<8,  /* sysfs */
	IFF_ALLMULTI			= 1<<9,  /* sysfs */
	IFF_MASTER			= 1<<10, /* __volatile__ */
	IFF_SLAVE			= 1<<11, /* __volatile__ */
	IFF_MULTICAST			= 1<<12, /* sysfs */
	IFF_PORTSEL			= 1<<13, /* sysfs */
	IFF_AUTOMEDIA			= 1<<14, /* sysfs */
	IFF_DYNAMIC			= 1<<15, /* sysfs */
	IFF_LOWER_UP			= 1<<16, /* __volatile__ */
	IFF_DORMANT			= 1<<17, /* __volatile__ */
	IFF_ECHO			= 1<<18, /* __volatile__ */
};
```

and generate the following code:

```rust
pub enum Name {
    IFF_UP = 0x1,
    IFF_BROADCAST = 0x2,
    IFF_DEBUG = 0x4,
    IFF_LOOPBACK = 0x8,
    IFF_POINTOPOINT = 0x10,
    IFF_NOTRAILERS = 0x20,
    IFF_RUNNING = 0x40,
    IFF_NOARP = 0x80,
    IFF_PROMISC = 0x100,
    IFF_ALLMULTI = 0x200,
    IFF_MASTER = 0x400,
    IFF_SLAVE = 0x800,
    IFF_MULTICAST = 0x1000,
    IFF_PORTSEL = 0x2000,
    IFF_AUTOMEDIA = 0x4000,
    IFF_DYNAMIC = 0x8000,
    IFF_LOWER_UP = 0x10000,
    IFF_DORMANT = 0x20000,
    IFF_ECHO = 0x40000,
}
impl ::std::str::FromStr for Name {
    type Err = ();
    #[allow(dead_code)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IFF_UP" => Ok(Name::IFF_UP),
            "IFF_BROADCAST" => Ok(Name::IFF_BROADCAST),
            "IFF_DEBUG" => Ok(Name::IFF_DEBUG),
            "IFF_LOOPBACK" => Ok(Name::IFF_LOOPBACK),
            "IFF_POINTOPOINT" => Ok(Name::IFF_POINTOPOINT),
            "IFF_NOTRAILERS" => Ok(Name::IFF_NOTRAILERS),
            "IFF_RUNNING" => Ok(Name::IFF_RUNNING),
            "IFF_NOARP" => Ok(Name::IFF_NOARP),
            "IFF_PROMISC" => Ok(Name::IFF_PROMISC),
            "IFF_ALLMULTI" => Ok(Name::IFF_ALLMULTI),
            "IFF_MASTER" => Ok(Name::IFF_MASTER),
            "IFF_SLAVE" => Ok(Name::IFF_SLAVE),
            "IFF_MULTICAST" => Ok(Name::IFF_MULTICAST),
            "IFF_PORTSEL" => Ok(Name::IFF_PORTSEL),
            "IFF_AUTOMEDIA" => Ok(Name::IFF_AUTOMEDIA),
            "IFF_DYNAMIC" => Ok(Name::IFF_DYNAMIC),
            "IFF_LOWER_UP" => Ok(Name::IFF_LOWER_UP),
            "IFF_DORMANT" => Ok(Name::IFF_DORMANT),
            "IFF_ECHO" => Ok(Name::IFF_ECHO),
            _ => Err( () )
        }
    }
}
impl Default for Name {
    fn default() -> Name {
        Name::IFF_UP
    }
}
impl ::std::fmt::Display for Name {
    #[allow(dead_code)]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Name::IFF_UP => write!(f, "IFF_UP"),
            Name::IFF_BROADCAST => write!(f, "IFF_BROADCAST"),
            Name::IFF_DEBUG => write!(f, "IFF_DEBUG"),
            Name::IFF_LOOPBACK => write!(f, "IFF_LOOPBACK"),
            Name::IFF_POINTOPOINT => write!(f, "IFF_POINTOPOINT"),
            Name::IFF_NOTRAILERS => write!(f, "IFF_NOTRAILERS"),
            Name::IFF_RUNNING => write!(f, "IFF_RUNNING"),
            Name::IFF_NOARP => write!(f, "IFF_NOARP"),
            Name::IFF_PROMISC => write!(f, "IFF_PROMISC"),
            Name::IFF_ALLMULTI => write!(f, "IFF_ALLMULTI"),
            Name::IFF_MASTER => write!(f, "IFF_MASTER"),
            Name::IFF_SLAVE => write!(f, "IFF_SLAVE"),
            Name::IFF_MULTICAST => write!(f, "IFF_MULTICAST"),
            Name::IFF_PORTSEL => write!(f, "IFF_PORTSEL"),
            Name::IFF_AUTOMEDIA => write!(f, "IFF_AUTOMEDIA"),
            Name::IFF_DYNAMIC => write!(f, "IFF_DYNAMIC"),
            Name::IFF_LOWER_UP => write!(f, "IFF_LOWER_UP"),
            Name::IFF_DORMANT => write!(f, "IFF_DORMANT"),
            Name::IFF_ECHO => write!(f, "IFF_ECHO"),
        }
    }
}
impl ::num::traits::FromPrimitive for Name {
    #[allow(dead_code)]
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            0x1 => Some(Name::IFF_UP),
            0x2 => Some(Name::IFF_BROADCAST),
            0x4 => Some(Name::IFF_DEBUG),
            0x8 => Some(Name::IFF_LOOPBACK),
            0x10 => Some(Name::IFF_POINTOPOINT),
            0x20 => Some(Name::IFF_NOTRAILERS),
            0x40 => Some(Name::IFF_RUNNING),
            0x80 => Some(Name::IFF_NOARP),
            0x100 => Some(Name::IFF_PROMISC),
            0x200 => Some(Name::IFF_ALLMULTI),
            0x400 => Some(Name::IFF_MASTER),
            0x800 => Some(Name::IFF_SLAVE),
            0x1000 => Some(Name::IFF_MULTICAST),
            0x2000 => Some(Name::IFF_PORTSEL),
            0x4000 => Some(Name::IFF_AUTOMEDIA),
            0x8000 => Some(Name::IFF_DYNAMIC),
            0x10000 => Some(Name::IFF_LOWER_UP),
            0x20000 => Some(Name::IFF_DORMANT),
            0x40000 => Some(Name::IFF_ECHO),
            _ => None
        }
    }
    #[allow(dead_code)]
    fn from_u64(n: u64) -> Option<Self> {
        match n {
            0x1 => Some(Name::IFF_UP),
            0x2 => Some(Name::IFF_BROADCAST),
            0x4 => Some(Name::IFF_DEBUG),
            0x8 => Some(Name::IFF_LOOPBACK),
            0x10 => Some(Name::IFF_POINTOPOINT),
            0x20 => Some(Name::IFF_NOTRAILERS),
            0x40 => Some(Name::IFF_RUNNING),
            0x80 => Some(Name::IFF_NOARP),
            0x100 => Some(Name::IFF_PROMISC),
            0x200 => Some(Name::IFF_ALLMULTI),
            0x400 => Some(Name::IFF_MASTER),
            0x800 => Some(Name::IFF_SLAVE),
            0x1000 => Some(Name::IFF_MULTICAST),
            0x2000 => Some(Name::IFF_PORTSEL),
            0x4000 => Some(Name::IFF_AUTOMEDIA),
            0x8000 => Some(Name::IFF_DYNAMIC),
            0x10000 => Some(Name::IFF_LOWER_UP),
            0x20000 => Some(Name::IFF_DORMANT),
            0x40000 => Some(Name::IFF_ECHO),
            _ => None
        }
    }
}
impl Name {
    fn pretty_fmt(f: &mut ::std::fmt::Formatter, flags: u32) -> ::std::fmt::Result {
        let mut shift: u32 = 0;
        let mut result: u32 = 1<<shift;
        let mut found = false;
        while result <= Name::IFF_ECHO as u32 {
            let tmp = result & flags;
            if tmp > 0 {
                if found {
                    try!(write!(f, "|"));
                }
                let flag = Name::from_u32(tmp).unwrap();
                try!(write!(f, "{}", flag));
                found = true;
            }
            shift += 1;
            result = 1<<shift;
        }
        write!(f, "")
    }
}pub enum Name {
    IFF_UP = 0x1,
    IFF_BROADCAST = 0x2,
    IFF_DEBUG = 0x4,
    IFF_LOOPBACK = 0x8,
    IFF_POINTOPOINT = 0x10,
    IFF_NOTRAILERS = 0x20,
    IFF_RUNNING = 0x40,
    IFF_NOARP = 0x80,
    IFF_PROMISC = 0x100,
    IFF_ALLMULTI = 0x200,
    IFF_MASTER = 0x400,
    IFF_SLAVE = 0x800,
    IFF_MULTICAST = 0x1000,
    IFF_PORTSEL = 0x2000,
    IFF_AUTOMEDIA = 0x4000,
    IFF_DYNAMIC = 0x8000,
    IFF_LOWER_UP = 0x10000,
    IFF_DORMANT = 0x20000,
    IFF_ECHO = 0x40000,
}
impl ::std::str::FromStr for Name {
    type Err = ();
    #[allow(dead_code)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IFF_UP" => Ok(Name::IFF_UP),
            "IFF_BROADCAST" => Ok(Name::IFF_BROADCAST),
            "IFF_DEBUG" => Ok(Name::IFF_DEBUG),
            "IFF_LOOPBACK" => Ok(Name::IFF_LOOPBACK),
            "IFF_POINTOPOINT" => Ok(Name::IFF_POINTOPOINT),
            "IFF_NOTRAILERS" => Ok(Name::IFF_NOTRAILERS),
            "IFF_RUNNING" => Ok(Name::IFF_RUNNING),
            "IFF_NOARP" => Ok(Name::IFF_NOARP),
            "IFF_PROMISC" => Ok(Name::IFF_PROMISC),
            "IFF_ALLMULTI" => Ok(Name::IFF_ALLMULTI),
            "IFF_MASTER" => Ok(Name::IFF_MASTER),
            "IFF_SLAVE" => Ok(Name::IFF_SLAVE),
            "IFF_MULTICAST" => Ok(Name::IFF_MULTICAST),
            "IFF_PORTSEL" => Ok(Name::IFF_PORTSEL),
            "IFF_AUTOMEDIA" => Ok(Name::IFF_AUTOMEDIA),
            "IFF_DYNAMIC" => Ok(Name::IFF_DYNAMIC),
            "IFF_LOWER_UP" => Ok(Name::IFF_LOWER_UP),
            "IFF_DORMANT" => Ok(Name::IFF_DORMANT),
            "IFF_ECHO" => Ok(Name::IFF_ECHO),
            _ => Err( () )
        }
    }
}
impl Default for Name {
    fn default() -> Name {
        Name::IFF_UP
    }
}
impl ::std::fmt::Display for Name {
    #[allow(dead_code)]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Name::IFF_UP => write!(f, "IFF_UP"),
            Name::IFF_BROADCAST => write!(f, "IFF_BROADCAST"),
            Name::IFF_DEBUG => write!(f, "IFF_DEBUG"),
            Name::IFF_LOOPBACK => write!(f, "IFF_LOOPBACK"),
            Name::IFF_POINTOPOINT => write!(f, "IFF_POINTOPOINT"),
            Name::IFF_NOTRAILERS => write!(f, "IFF_NOTRAILERS"),
            Name::IFF_RUNNING => write!(f, "IFF_RUNNING"),
            Name::IFF_NOARP => write!(f, "IFF_NOARP"),
            Name::IFF_PROMISC => write!(f, "IFF_PROMISC"),
            Name::IFF_ALLMULTI => write!(f, "IFF_ALLMULTI"),
            Name::IFF_MASTER => write!(f, "IFF_MASTER"),
            Name::IFF_SLAVE => write!(f, "IFF_SLAVE"),
            Name::IFF_MULTICAST => write!(f, "IFF_MULTICAST"),
            Name::IFF_PORTSEL => write!(f, "IFF_PORTSEL"),
            Name::IFF_AUTOMEDIA => write!(f, "IFF_AUTOMEDIA"),
            Name::IFF_DYNAMIC => write!(f, "IFF_DYNAMIC"),
            Name::IFF_LOWER_UP => write!(f, "IFF_LOWER_UP"),
            Name::IFF_DORMANT => write!(f, "IFF_DORMANT"),
            Name::IFF_ECHO => write!(f, "IFF_ECHO"),
        }
    }
}
impl ::num::traits::FromPrimitive for Name {
    #[allow(dead_code)]
    fn from_i64(n: i64) -> Option<Self> {
        match n {
            0x1 => Some(Name::IFF_UP),
            0x2 => Some(Name::IFF_BROADCAST),
            0x4 => Some(Name::IFF_DEBUG),
            0x8 => Some(Name::IFF_LOOPBACK),
            0x10 => Some(Name::IFF_POINTOPOINT),
            0x20 => Some(Name::IFF_NOTRAILERS),
            0x40 => Some(Name::IFF_RUNNING),
            0x80 => Some(Name::IFF_NOARP),
            0x100 => Some(Name::IFF_PROMISC),
            0x200 => Some(Name::IFF_ALLMULTI),
            0x400 => Some(Name::IFF_MASTER),
            0x800 => Some(Name::IFF_SLAVE),
            0x1000 => Some(Name::IFF_MULTICAST),
            0x2000 => Some(Name::IFF_PORTSEL),
            0x4000 => Some(Name::IFF_AUTOMEDIA),
            0x8000 => Some(Name::IFF_DYNAMIC),
            0x10000 => Some(Name::IFF_LOWER_UP),
            0x20000 => Some(Name::IFF_DORMANT),
            0x40000 => Some(Name::IFF_ECHO),
            _ => None
        }
    }
    #[allow(dead_code)]
    fn from_u64(n: u64) -> Option<Self> {
        match n {
            0x1 => Some(Name::IFF_UP),
            0x2 => Some(Name::IFF_BROADCAST),
            0x4 => Some(Name::IFF_DEBUG),
            0x8 => Some(Name::IFF_LOOPBACK),
            0x10 => Some(Name::IFF_POINTOPOINT),
            0x20 => Some(Name::IFF_NOTRAILERS),
            0x40 => Some(Name::IFF_RUNNING),
            0x80 => Some(Name::IFF_NOARP),
            0x100 => Some(Name::IFF_PROMISC),
            0x200 => Some(Name::IFF_ALLMULTI),
            0x400 => Some(Name::IFF_MASTER),
            0x800 => Some(Name::IFF_SLAVE),
            0x1000 => Some(Name::IFF_MULTICAST),
            0x2000 => Some(Name::IFF_PORTSEL),
            0x4000 => Some(Name::IFF_AUTOMEDIA),
            0x8000 => Some(Name::IFF_DYNAMIC),
            0x10000 => Some(Name::IFF_LOWER_UP),
            0x20000 => Some(Name::IFF_DORMANT),
            0x40000 => Some(Name::IFF_ECHO),
            _ => None
        }
    }
}
impl Name {
    fn pretty_fmt(f: &mut ::std::fmt::Formatter, flags: u32) -> ::std::fmt::Result {
        let mut shift: u32 = 0;
        let mut result: u32 = 1<<shift;
        let mut found = false;
        while result <= Name::IFF_ECHO as u32 {
            let tmp = result & flags;
            if tmp > 0 {
                if found {
                    try!(write!(f, "|"));
                }
                let flag = Name::from_u32(tmp).unwrap();
                try!(write!(f, "{}", flag));
                found = true;
            }
            shift += 1;
            result = 1<<shift;
        }
        write!(f, "")
    }
}
```

You can choose to have rust-enum-derive implement all, some, or none of the
methods/traits.
