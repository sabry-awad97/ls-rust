use std::{error::Error, path::Path};

mod args {
    use std::error::Error;
    use std::str::FromStr;
    use structopt::StructOpt;

    #[derive(StructOpt, Debug)]
    #[structopt(
        name = "ls",
        author = "Sabry <dr.sabry@gmail.com>",
        version = "0.1.0",
        about = "Rust ls"
    )]
    pub struct Arguments {
        #[structopt(
            short = "a",
            long = "all",
            help = "Show all files and directories, including hidden ones (those that start with a dot)."
        )]
        pub show_hidden: bool,

        #[structopt(
            short = "A",
            long = "almost-all",
            help = "Like -a, but do not include the . and .. directories"
        )]
        pub show_almost_all: bool,

        #[structopt(
            short = "b",
            long = "escape",
            help = "Show octal escapes for nongraphic characters"
        )]
        pub escape: bool,

        #[structopt(
            name("time"),
            value_names(&["WHEN"]),
            short("c"),
            long("time"),
            help("Use time as sort key instead of name"),
            possible_values = &["mtime", "atime", "ctime"]
        )]
        pub time: Option<TimeSort>,

        #[structopt(
            short = "F",
            long = "classify",
            help = "Append a character to each file name indicating the file type"
        )]
        pub classify: bool,

        #[structopt(
            short = "d",
            long = "max-depth",
            help = "Limit the components of the path"
        )]
        pub max_depth: Option<usize>,

        #[structopt(
            short = "l",
            long = "limit",
            help = "Limit the number of entries displayed"
        )]
        pub limit: Option<usize>,

        #[structopt(name = "path", help = "The path to list", index = 1)]
        pub path: Option<String>,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum TimeSort {
        Atime,
        Mtime,
        Ctime,
    }

    impl FromStr for TimeSort {
        type Err = Box<dyn Error>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "atime" => Ok(TimeSort::Atime),
                "mtime" => Ok(TimeSort::Mtime),
                "ctime" => Ok(TimeSort::Ctime),
                _ => Err(format!("invalid argument '{}' for '-c' option", s).into()),
            }
        }
    }

    pub fn parse_args() -> Result<Arguments, Box<dyn Error>> {
        Ok(Arguments::from_args())
    }
}

mod entries {
    use std::error::Error;
    use std::fs;
    use std::fs::DirEntry;
    use std::path::Path;

    pub fn read_entries(
        path: &Path,
        show_almost_all: bool,
        max_depth: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<DirEntry>, Box<dyn Error>> {
        let mut entries: Vec<DirEntry> = fs::read_dir(path)?
            .filter_map(|res| res.ok())
            .filter(|entry| {
                if show_almost_all {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| s != "." && s != "..")
                        .unwrap_or(true)
                } else {
                    !entry
                        .file_name()
                        .to_str()
                        .map(|s| s.starts_with("."))
                        .unwrap_or(false)
                }
            })
            .take(limit.unwrap_or_else(|| std::usize::MAX))
            .collect();

        if let Some(max_depth) = max_depth {
            let mut i = 0;
            while i < entries.len() {
                let entry = &entries[i];
                if entry.path().components().count() > max_depth {
                    entries.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        Ok(entries)
    }
}

mod list {
    use chrono::offset::Utc;
    use chrono::DateTime;
    use std::error::Error;
    use std::fs::DirEntry;

    use crate::args::TimeSort;

    pub fn list_dir(
        entries: &[DirEntry],
        escape: bool,
        time: Option<TimeSort>,
        classify: bool,
    ) -> Result<(), Box<dyn Error>> {
        for entry in entries {
            let path = entry.path();
            let mut components = path.components();
            let file_name = components
                .next_back()
                .unwrap()
                .as_os_str()
                .to_string_lossy();

            if escape {
                print!("{}", escape_string(&file_name));
            } else {
                print!("{}", file_name);
            }

            if classify {
                let file_type = match entry.file_type()? {
                    t if t.is_dir() => '/',
                    t if t.is_symlink() => '@',
                    t if t.is_file() => ' ',
                    _ => ' ',
                };
                print!("{}", file_type);
            }

            if let Some(time) = time {
                let metadata = entry.metadata()?;

                let access_time: DateTime<Utc> = metadata.accessed()?.into();
                let modified_time: DateTime<Utc> = metadata.modified()?.into();
                let created_time: DateTime<Utc> = metadata.created()?.into();

                let time_string = match time {
                    TimeSort::Atime => access_time.format("%b %e %R").to_string(),
                    TimeSort::Mtime => modified_time.format("%b %e %R").to_string(),
                    TimeSort::Ctime => created_time.format("%b %e %R").to_string(),
                };
                print!("  {}", time_string);
            }

            println!();
        }

        Ok(())
    }

    fn escape_string(s: &str) -> String {
        let mut escaped = String::new();
        for c in s.chars() {
            if c.is_ascii_graphic() {
                escaped.push(c);
            } else {
                escaped.push_str(&format!("\\{:03o}", c as u8));
            }
        }
        escaped
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;
    let path = args.path.unwrap_or_else(|| ".".to_string());
    let path = Path::new(&path);

    let entries = entries::read_entries(path, args.show_almost_all, args.max_depth, args.limit)?;
    list::list_dir(&entries, args.escape, args.time, args.classify)?;

    Ok(())
}
