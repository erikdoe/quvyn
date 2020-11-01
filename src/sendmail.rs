use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::io::{Error,ErrorKind};
use std::io::BufReader;

// Part of this file copyright (c) 2015 Alexander
// https://github.com/vokeio/rust-sendmail

pub fn send(to_address: &str, subject_text: &str, body_text: &str) -> Result<(),Error>
{
    let mut cmd = Command::new("sendmail");
    cmd.args(&[&"-t"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut process = cmd.spawn()?;

    { // required because of process.wait()
        let mut stdin = process.stdin.as_mut().ok_or(Error::new(ErrorKind::BrokenPipe, "no stdin"))?;

        writeln!(&mut stdin,"To: {}", to_address)?;
        writeln!(&mut stdin,"Subject: {}",subject_text)?;
        writeln!(&mut stdin,"" )?;
        stdin.write_all(body_text.as_bytes())?;
        stdin.flush()?;
    }

    process.wait()?;

    let mut stderr_buffer = BufReader::new(process.stderr.ok_or(Error::new(ErrorKind::BrokenPipe, "no stderr"))?);
    let mut stderr: String = String::new();
    stderr_buffer.read_to_string(&mut stderr)?;

    if stderr.is_empty() {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, format!("sendmail returned: {}",stderr)))
    }
}
