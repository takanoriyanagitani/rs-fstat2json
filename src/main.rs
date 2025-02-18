use std::process::ExitCode;

use std::io;

use rs_fstat2json::stat::least::stdin2filenames2stats2stdout;

fn sub() -> Result<(), io::Error> {
    stdin2filenames2stats2stdout()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
