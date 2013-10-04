extern mod extra;

use extra::json;
use extra::flatpipes::flatteners::FromReader;
use extra::getopts::groups::*;

use extra::serialize::{Decodable, Encodable};

#[deriving(Encodable,Decodable)]
/// The configuration for a crate that will be built.
struct CrateConfig {
    name: ~str,
    repo: ~str,
    author: ~str,
    commands: Option<~[Command]>,
    crate_root: ~str,
    description: ~str,
}

#[deriving(Clone,Encodable,Decodable)]
/// A command to run when document
struct Command {
    env: Option<~[(~str, ~str)]>,
    program: ~str,
    args: ~[~str]
}

type Config = ~[CrateConfig];

fn main() {
    let opts = ~[
        reqopt("", "config", "JSON config file", "FILE"),
    ];

    let args = match getopts(std::os::args(), opts) {
        Ok(x) => x,
        Err(y) => { println(usage(y.to_err_msg(), opts)); fail!() },
    };

    let path = Path(args.opt_str("config").unwrap());
    let mut decoder: json::Decoder = FromReader::from_reader(std::io::file_reader(&path).unwrap());

    let config: Config = Decodable::decode(&mut decoder);

    for crate in config.iter() {
        let p = Path(crate.name);
        if !p.exists() {
            assert!(run("git", [~"clone", crate.repo.clone(), crate.name.clone()], None, None));
        }
        let cmds = crate.commands.clone().unwrap_or(~[]);
        for command in cmds.iter() {
            assert!(run(command.program, command.args, Some(&p), command.env.clone()));
        }
        assert!(run("rustdoc", [crate.crate_root.clone()], None, None));
    }

    build_index(config);
}

fn build_index(c: Config) {
    use std::rt::io::*;
    let mut f = file::open(&Path("doc/index.html"), CreateOrTruncate, ReadWrite);
    f.write(bytes!("<!doctype html>
    <html><head><title>Rust Library Documentation</title></head><body><ul>\n"));
    for crate in c.move_iter() {
        let s = format!("<li><a href=\"./{}\">{}</a> - {}</li>\n", crate.name, crate.name, crate.description);
        f.write(s.as_bytes());
    }
    f.write(bytes!("</ul></body></html>"));
}

fn run(prog: &str, args: &[~str], workdir: Option<&Path>, env: Option<~[(~str, ~str)]>) -> bool {
    use std::run::{Process, ProcessOptions};
    use std::str::from_utf8;

    let opts = ProcessOptions { env: env, dir: workdir, ..ProcessOptions::new() };
    let out = Process::new(prog, args, opts).finish_with_output();
    if out.status != 0 {
        error2!("{} {:?} returned {}", prog, args, out.status);
        info2!("stdout: {}", from_utf8(out.output));
        info2!("stderr: {}", from_utf8(out.error));
        return false;
    }
    true
}
