use crate::io::{self, ReadFunction};
use crate::methods::{self, StepMethod};
use clap::{crate_authors, App, Arg, ArgMatches};

/// Indicates how many reference files are passed
#[derive(Clone)]
pub enum Reference {
    One(String),
    Two(String, String),
    None,
}

/// Create a container for dealing with clap and being able to test arg parsing
pub enum ClapApp {
    App,
}

impl ClapApp {
    /// Create and return the clap::App
    pub fn get(&self) -> App {
        App::new("Multi-threaded Bader Charge Analysis")
            .author(crate_authors!())
            .version("0.1.0")
            .arg(Arg::with_name("file")
                .required(true)
                .index(1))
            .arg(Arg::with_name("method")
                .short('m')
                .long("method")
                .takes_value(true)
                .possible_value("ongrid")
                .possible_value("neargrid")
                .case_insensitive(false)
                .about("method by which to partition the charge density")
                .long_about(
"Use the \"near-grid\" or \"on-grid\" methods based on the algorithms presented
in W. Tang et al. A grid-based Bader analysis algorithm without lattice bias,
J. Phys.: Condens. Matter 21, 084204 (2009)"))
            .arg(Arg::with_name("file type")
                .short('t')
                .long("type")
                .takes_value(true)
                .possible_value("cube")
                .possible_value("vasp")
                .case_insensitive(false)
                .about("the file type of the charge density")
                .long_about(
"The file type of the input file. If this is not supplied the type will attempt
to be infered from the filename"))
            .arg(Arg::with_name("reference")
                .short('r')
                .long("ref")
                .multiple(true)
                .max_values(2)
                .number_of_values(1)
                .about("file(s) containing reference charge")
                .long_about(
"A reference charge to do the partitioning upon. Two files can be passed
by using multiple flags (bader CHGCAR -r AECCAR0 -r AECCAR2). If two files are
passed they are summed together."))
            .arg(Arg::with_name("all electron")
                .short('a')
                .long("aec")
                .about("convience flag for reading both aeccars")
                .takes_value(false)
                .multiple(false)
                .conflicts_with("reference"))
            .arg(Arg::with_name("vacuum tolerance")
                .short('v')
                .long("vac")
                .takes_value(true)
                .about("cut-off at which charge is considered vacuum")
                .long_about(
"Values of density below the supplied value are considered vacuum and are not
included in the calculation. A value of \"auto\" can be passed to use 1E-3 C*m^-3"))
            .arg(Arg::with_name("threads")
                .short('J')
                .long("threads")
                .takes_value(true)
                .default_value("0")
                .about("Number of threads to distribute the calculation over.")
                .long_about(
"The number of threads to be used by the program. A default value of 0 is used
to allow the program to best decide how to use the available hardware."))
    }
}

/// Holds the arguments passed to the program from the command-line
pub struct Args {
    pub file: String,
    pub read: ReadFunction,
    pub method: StepMethod,
    pub reference: Reference,
    pub threads: usize,
    pub vacuum_tolerance: Option<f64>,
    pub zyx_format: bool,
}

impl Args {
    /// Initialises the structure from the command-line arguments.
    pub fn new(arguments: ArgMatches) -> Self {
        let file = match arguments.value_of("file") {
            Some(f) => String::from(f),
            None => String::new(),
        };
        let file_type = match arguments.value_of("file type") {
            Some(f) => Some(String::from(f)),
            None => None,
        };
        let mut zyx_format = false;
        let read: ReadFunction = match file_type {
            Some(ftype) => {
                if ftype.eq("cube") {
                    io::cube::read
                } else {
                    zyx_format = true;
                    io::vasp::read
                }
            }
            None => {
                if file.to_lowercase().contains("cube") {
                    io::cube::read
                } else if file.to_lowercase().contains("car") {
                    zyx_format = true;
                    io::vasp::read
                } else {
                    println!("Error: File-type cannot be infered, attempting to read as VASP");
                    zyx_format = true;
                    io::vasp::read
                }
            }
        };
        let method = match arguments.value_of("method") {
            Some("neargrid") => methods::neargrid,
            Some("ongrid") => methods::ongrid,
            Some(_) => methods::neargrid,
            None => methods::neargrid,
        };
        // safe to unwrap as threads has a default value of 0
        let threads =
            match arguments.value_of("threads").unwrap().parse::<usize>() {
                Ok(x) => x,
                Err(e) => panic!("Couldn't parse threads into integer:\n{}", e),
            };
        let vacuum_tolerance = match arguments.value_of("vacuum tolerance") {
            Some(s) => {
                if s.eq("auto") {
                    Some(1E-3)
                } else {
                    match s.parse::<f64>() {
                        Ok(x) => Some(x),
                        Err(e) => {
                            panic!("Couldn't parse vacuum tolerance into float:\n{}", e)
                        }
                    }
                }
            }
            None => None,
        };
        let references: Vec<_> = if arguments.is_present("all electron") {
            if read as usize != io::vasp::read as usize {
                panic!("Error: Cannot use AECCAR flag for non VASP file-types.")
            }
            vec!["AECCAR0", "AECCAR2"]
        } else {
            match arguments.values_of("reference") {
                Some(x) => x.collect(),
                None => Vec::with_capacity(0),
            }
        };
        let reference = match references.len() {
            2 => Reference::Two(String::from(references[0]),
                                String::from(references[1])),
            1 => Reference::One(String::from(references[0])),
            _ => Reference::None,
        };
        return Self { file,
                      read,
                      method,
                      reference,
                      threads,
                      vacuum_tolerance,
                      zyx_format };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clapapp_get() {
        let app = ClapApp::App.get();
        assert_eq!(app.get_name(), "Multi-threaded Bader Charge Analysis")
    }

    #[test]
    fn argument_file() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader", "CHGCAR"]);
        let args = Args::new(matches);
        assert_eq!(args.file, String::from("CHGCAR"));
    }

    #[test]
    #[should_panic]
    fn argument_no_file() {
        let app = ClapApp::App.get();
        let _ = app.try_get_matches_from(vec!["bader"])
                   .unwrap_or_else(|e| panic!("An error occurs: {}", e));
    }

    #[test]
    fn argument_method_ongrid() {
        let app = ClapApp::App.get();
        let matches =
            app.get_matches_from(vec!["bader", "CHGCAR", "-m", "ongrid"]);
        let args = Args::new(matches);
        let method: StepMethod = methods::ongrid;
        assert_eq!(args.method as usize, method as usize);
    }

    #[test]
    fn argument_method_neargrid() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader", "CHGCAR",
                                                "--method", "neargrid"]);
        let args = Args::new(matches);
        let method: StepMethod = methods::neargrid;
        assert_eq!(args.method as usize, method as usize);
    }

    #[test]
    fn argument_method_default() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader", "CHGCAR"]);
        let args = Args::new(matches);
        let method: StepMethod = methods::neargrid;
        assert_eq!(args.method as usize, method as usize);
    }

    #[test]
    #[should_panic]
    fn argument_method_not_method() {
        let app = ClapApp::App.get();
        let _ = app.try_get_matches_from(vec!["bader", "CHGCAR", "-m",
                                              "ngrid"])
                   .unwrap_or_else(|e| panic!("An error occurs: {}", e));
    }

    #[test]
    fn argument_file_type_default_vasp() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader", "CHGCAR"]);
        let args = Args::new(matches);
        let read: ReadFunction = io::vasp::read;
        assert_eq!(args.read as usize, read as usize);
    }

    #[test]
    fn argument_file_type_default_unknown() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader", "CHG"]);
        let args = Args::new(matches);
        let read: ReadFunction = io::vasp::read;
        assert_eq!(args.read as usize, read as usize);
    }

    #[test]
    fn argument_file_type_vasp() {
        let app = ClapApp::App.get();
        let matches =
            app.get_matches_from(vec!["bader", "CHGCAR", "-t", "vasp"]);
        let args = Args::new(matches);
        let read: ReadFunction = io::vasp::read;
        assert_eq!(args.read as usize, read as usize);
    }

    #[test]
    fn argument_file_type_default_cube() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader", "charge.cube"]);
        let args = Args::new(matches);
        let read: ReadFunction = io::cube::read;
        assert_eq!(args.read as usize, read as usize);
    }

    #[test]
    fn argument_file_type_cube() {
        let app = ClapApp::App.get();
        let matches = app.get_matches_from(vec!["bader",
                                                "charge.cube",
                                                "--type",
                                                "cube"]);
        let args = Args::new(matches);
        let read: ReadFunction = io::cube::read;
        assert_eq!(args.read as usize, read as usize);
    }

    #[test]
    #[should_panic]
    fn argument_file_type_not_type() {
        let app = ClapApp::App.get();
        let _ = app.try_get_matches_from(vec!["bader", "CHGCAR", "-t", "basp"])
                   .unwrap_or_else(|e| panic!("An error occurs: {}", e));
    }

    #[test]
    fn argument_reference_one() {
        let app = ClapApp::App.get();
        let matches =
            app.get_matches_from(vec!["bader", "CHGCAR", "-r", "CHGCAR_sum"]);
        let args = Args::new(matches);
        let flag = match args.reference {
            Reference::One(_) => true,
            _ => false,
        };
        assert!(flag)
    }

    #[test]
    fn argument_reference_two() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "-r", "AECCAR0", "--ref", "AECCAR2"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        let flag = match args.reference {
            Reference::Two(_, _) => true,
            _ => false,
        };
        assert!(flag)
    }

    #[test]
    fn argument_reference_none() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        let flag = match args.reference {
            Reference::None => true,
            _ => false,
        };
        assert!(flag)
    }

    #[test]
    fn argument_aeccar() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "-a"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        let flag = match args.reference {
            Reference::Two(x, y) => {
                if (x == String::from("AECCAR0"))
                   && (y == String::from("AECCAR2"))
                {
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        assert!(flag)
    }

    #[test]
    #[should_panic]
    fn argument_aeccar_cube() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "charge.cube", "-a"];
        let matches = app.get_matches_from(v);
        let _ = Args::new(matches);
    }

    #[test]
    fn argument_vacuum_tolerance_auto() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "-v", "auto"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        assert_eq!(args.vacuum_tolerance, Some(1E-3))
    }

    #[test]
    fn argument_vacuum_tolerance_float() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "--vac", "1E-4"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        assert_eq!(args.vacuum_tolerance, Some(1E-4))
    }

    #[test]
    #[should_panic]
    fn argument_vacuum_tolerance_not_float() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "-v", "0.00.1"];
        let matches = app.get_matches_from(v);
        let _ = Args::new(matches);
    }

    #[test]
    fn argument_threads_default() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        assert_eq!(args.threads, 0)
    }

    #[test]
    fn argument_threads_int() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "--threads", "1"];
        let matches = app.get_matches_from(v);
        let args = Args::new(matches);
        assert_eq!(args.threads, 1)
    }

    #[test]
    #[should_panic]
    fn argument_threads_not_int() {
        let app = ClapApp::App.get();
        let v = vec!["bader", "CHGCAR", "-J", "0.1"];
        let matches = app.get_matches_from(v);
        let _ = Args::new(matches);
    }
}