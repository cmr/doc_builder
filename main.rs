#[feature(globs)];
extern mod extra;

use std::rt::io;
use std::rt::io::Reader;
use std::rt::io::{Open, Read};

use extra::json;
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

    let path = Path::new(args.opt_str("config").unwrap());
    let mut file = io::file::open(&path, Open, Read).unwrap();
    let json = json::from_reader(&mut file as &mut Reader).unwrap();
    let mut decoder = json::Decoder(json);
    let config: Config = Decodable::decode(&mut decoder);

    for crate in config.iter() {
        let p = Path::new(crate.name.clone());
        if !p.exists() {
            assert!(run("git", [~"clone", crate.repo.clone(), crate.name.clone()], None, None));
        } else {
            assert!(run("git", [~"pull", ~"origin", ~"master"], Some(&p), None));
        }
        let cmds = crate.commands.clone().unwrap_or(~[]);
        for command in cmds.iter() {
            if !run(command.program, command.args, Some(&p), command.env.clone()) {
                error!("Warning: building {} failed running {}", crate.name, command.program);
                continue;
            }
        }
        if !run("rustdoc", [crate.name + "/" + crate.crate_root.clone()], None, None) {
            error!("Warning: documenting {} failed", crate.name);
            continue;
        }
    }

    build_index(config);
}

fn build_index(c: Config) {
    use std::rt::io::*;
    let mut f = file::open(&Path::new("doc/index.html"), CreateOrTruncate, ReadWrite);
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
        error!("{} {:?} returned {}", prog, args, out.status);
        info!("stdout: {}", from_utf8(out.output));
        info!("stderr: {}", from_utf8(out.error));
        return false;
    }
    true
}
