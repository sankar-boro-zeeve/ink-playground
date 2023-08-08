// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains the compile service of the backend. It receives a
//! string of Rust source code and returns the result of compiling the code.
//! For security reason we run the compilation inside a Docker container.
//! In order to ease testing, the service is parameterized by a compile
//! strategy. This allows easy mocking.

mod build_command;
mod docker_command;

use crate::build_command::{
    build_compile_command,
    build_formatting_command,
    build_testing_command,
};
use serde::{
    Deserialize,
    Serialize,
};
use snafu::{
    OptionExt,
    ResultExt,
    Snafu,
};
use std::{
    ffi::OsStr,
    fs::{
        self,
        File,
    },
    io::{
        self,
        prelude::*,
        BufReader,
        ErrorKind,
    },
    os::unix::prelude::PermissionsExt,
    path::{
        Path,
        PathBuf,
    },
    string,
    time::Duration,
};
use tempdir::TempDir;
use tokio::process::Command;
use typescript_type_def::TypeDef;

// -------------------------------------------------------------------------------------------------
// TYPES
// -------------------------------------------------------------------------------------------------

pub struct Sandbox {
    #[allow(dead_code)]
    scratch: TempDir,
    input_file: PathBuf,
    output_dir: PathBuf,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to create temporary directory: {}", source))]
    UnableToCreateTempDirSnafu { source: io::Error },

    #[snafu(display("Unable to create output directory: {}", source))]
    UnableToCreateOutputDirSnafu { source: io::Error },

    #[snafu(display("Unable to set permissions for output directory: {}", source))]
    UnableToSetOutputPermissionsSnafu { source: io::Error },

    #[snafu(display("Unable to create source file: {}", source))]
    UnableToCreateSourceFileSnafu { source: io::Error },

    #[snafu(display("Unable to set permissions for source file: {}", source))]
    UnableToSetSourcePermissionsSnafu { source: io::Error },

    #[snafu(display("Output was not valid UTF-8: {}", source))]
    OutputNotUtf8 { source: string::FromUtf8Error },

    #[snafu(display("Unable to read output file: {}", source))]
    UnableToReadOutputSnafu { source: io::Error },

    #[snafu(display("Unable to start the compiler: {}", source))]
    UnableToStartCompilerSnafu { source: io::Error },

    #[snafu(display("Unable to find the compiler ID"))]
    MissingCompilerIdSnafu,

    #[snafu(display("Unable to wait for the compiler: {}", source))]
    UnableToWaitForCompilerSnafu { source: io::Error },

    #[snafu(display("Unable to get output from the compiler: {}", source))]
    UnableToGetOutputFromCompilerSnafu { source: io::Error },

    #[snafu(display("Unable to remove the compiler: {}", source))]
    UnableToRemoveCompilerSnafu { source: io::Error },

    #[snafu(display("Compiler execution took longer than {} ms", timeout.as_millis()))]
    CompilerExecutionTimedOutSnafu {
        source: tokio::time::error::Elapsed,
        timeout: Duration,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Deserialize, Serialize, TypeDef, Debug, Clone)]
pub struct CompilationRequest {
    pub source: String,
    pub version: String
}

#[derive(Deserialize, Serialize, TypeDef, PartialEq, Debug, Clone, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompilationResult {
    Success {
        wasm: Vec<u8>,
        stdout: String,
        stderr: String,
    },
    Error {
        stdout: String,
        stderr: String,
    },
}

#[derive(Deserialize, Serialize, TypeDef, Debug, Clone)]
pub struct TestingRequest {
    pub source: String,
}

#[derive(Deserialize, Serialize, TypeDef, PartialEq, Debug, Clone, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TestingResult {
    Success { stdout: String, stderr: String },
    Error { stdout: String, stderr: String },
}

#[derive(Deserialize, Serialize, TypeDef, Debug, Clone)]
pub struct FormattingRequest {
    pub source: String,
}

#[derive(Deserialize, Serialize, TypeDef, PartialEq, Debug, Clone, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormattingResult {
    Success { source: String, stderr: String },
    Error { stdout: String, stderr: String },
}

// -------------------------------------------------------------------------------------------------
// CONSTANTS
// -------------------------------------------------------------------------------------------------

const DOCKER_PROCESS_TIMEOUT_HARD: Duration = Duration::from_secs(60);

// -------------------------------------------------------------------------------------------------
// TRAIT IMPLEMENTATION
// -------------------------------------------------------------------------------------------------

impl Sandbox {
    pub fn new() -> Result<Self> {
        let scratch =
            TempDir::new("playground").context(UnableToCreateTempDirSnafuSnafu)?;
        let input_file = scratch.path().join("input.rs");
        let output_dir = scratch.path().join("output");
        fs::create_dir(&output_dir).context(UnableToCreateOutputDirSnafuSnafu)?;

        fs::set_permissions(&output_dir, wide_open_permissions())
            .context(UnableToSetOutputPermissionsSnafuSnafu)?;

        Ok(Sandbox {
            scratch,
            input_file,
            output_dir,
        })
    }

    pub fn compile(&self, req: &CompilationRequest) -> Result<CompilationResult> {
        self.write_source_code(&req.source)?;

        let command = build_compile_command(&self.input_file, &self.output_dir, &req.version);

        println!("Executing command: \n{:?}", command);

        let output = run_command_with_timeout(command)?;
        let file = fs::read_dir(&self.output_dir)
            .context(UnableToReadOutputSnafuSnafu)?
            .flatten()
            .map(|entry| entry.path())
            .find(|path| path.extension() == Some(OsStr::new("contract")));

        let stdout = vec_to_str(output.stdout)?;
        let stderr = vec_to_str(output.stderr)?;

        let compile_response = match file {
            Some(file) => {
                match read(&file) {
                    Ok(Some(wasm)) => {
                        CompilationResult::Success {
                            wasm,
                            stderr,
                            stdout,
                        }
                    }
                    Ok(None) => CompilationResult::Error { stderr, stdout },
                    Err(_) => CompilationResult::Error { stderr, stdout },
                }
            }
            None => CompilationResult::Error { stderr, stdout },
        };

        Ok(compile_response)
    }

    pub fn test(&self, req: &TestingRequest) -> Result<TestingResult> {
        self.write_source_code(&req.source)?;

        let command = build_testing_command(&self.input_file, "4_2_0");

        println!("Executing command: \n{:?}", command);

        let output = run_command_with_timeout(command)?;

        let stdout = vec_to_str(output.stdout)?;
        let stderr = vec_to_str(output.stderr)?;

        let testing_response = TestingResult::Success { stderr, stdout };

        Ok(testing_response)
    }

    pub fn format(&self, req: &FormattingRequest) -> Result<FormattingResult> {
        self.write_source_code(&req.source)?;

        let command = build_formatting_command(&self.input_file, "4_2_0");

        println!("Executing command: \n{:?}", command);

        let output = run_command_with_timeout(command)?;

        let stdout = vec_to_str(output.stdout)?;
        let stderr = vec_to_str(output.stderr)?;

        let formatting_response = FormattingResult::Success {
            stderr,
            source: stdout,
        };

        Ok(formatting_response)
    }

    fn write_source_code(&self, code: &str) -> Result<()> {
        fs::write(&self.input_file, code).context(UnableToCreateSourceFileSnafuSnafu)?;
        fs::set_permissions(&self.input_file, wide_open_permissions())
            .context(UnableToSetSourcePermissionsSnafuSnafu)?;

        println!(
            "Wrote {} bytes of source to {}",
            code.len(),
            self.input_file.display()
        );
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// UTIL FUNCTIONS
// -------------------------------------------------------------------------------------------------

fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    let f = match File::open(path) {
        Ok(f) => f,
        Err(ref e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        e => e.context(UnableToReadOutputSnafuSnafu)?,
    };
    let mut f = BufReader::new(f);
    let metadata = fs::metadata(path).expect("unable to read metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    Ok(Some(buffer))
}

#[tokio::main]
async fn run_command_with_timeout(mut command: Command) -> Result<std::process::Output> {
    use std::os::unix::process::ExitStatusExt;

    let timeout = DOCKER_PROCESS_TIMEOUT_HARD;
    println!("executing command!");
    let output = command
        .output()
        .await
        .context(UnableToStartCompilerSnafuSnafu)?;
    println!("Done! {:?}", output);
    // Exit early, in case we don't have the container
    // if !output.status.success() {
    // return Ok(output);
    // }
    // let response = &output.stdout;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let id = stdout
        .lines()
        .next()
        .context(MissingCompilerIdSnafuSnafu)?
        .trim();
    let stderr = &output.stderr;

    // ----------

    let mut command = docker_command!("wait", id);

    let timed_out = match tokio::time::timeout(timeout, command.output()).await {
        Ok(Ok(o)) => {
            // Didn't time out, didn't fail to run
            let o = String::from_utf8_lossy(&o.stdout);
            let code = o
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(i32::MAX);
            Ok(ExitStatusExt::from_raw(code))
        }
        Ok(e) => return e.context(UnableToWaitForCompilerSnafuSnafu), // Failed to run
        Err(e) => Err(e),                                             // Timed out
    };

    // ----------

    let mut command = docker_command!("logs", id);
    let mut output = command
        .output()
        .await
        .context(UnableToGetOutputFromCompilerSnafuSnafu)?;

    // ----------

    let mut command = docker_command!(
        "rm", // Kills container if still running
        "--force", id
    );
    command.stdout(std::process::Stdio::null());
    command
        .status()
        .await
        .context(UnableToRemoveCompilerSnafuSnafu)?;

    let code = timed_out.context(CompilerExecutionTimedOutSnafuSnafu { timeout })?;

    output.status = code;
    output.stderr = stderr.to_owned();

    Ok(output)
}

// We must create a world-writable files (rustfmt) and directories
// (LLVM IR) so that the process inside the Docker container can write
// into it.
//
// This problem does *not* occur when using the indirection of
// docker-machine.
fn wide_open_permissions() -> std::fs::Permissions {
    PermissionsExt::from_mode(0o777)
}

fn vec_to_str(v: Vec<u8>) -> Result<String> {
    String::from_utf8(v).context(OutputNotUtf8Snafu)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_check(source: String, version: String) -> Option<bool> {
        Sandbox::new()
            .and_then(|sandbox| sandbox.compile(&CompilationRequest { source, version }))
            .map(|result| {
                match result {
                    CompilationResult::Success {
                        wasm,
                        stdout: _,
                        stderr: _,
                    } => !wasm.is_empty(),
                    CompilationResult::Error {
                        stdout: _,
                        stderr: _,
                    } => false,
                }
            })
            .ok()
    }

    #[test]
    fn test_compile_valid_code() {
        let flipper_code = include_str!("../../contract/lib.rs");
        let actual_result = compile_check(flipper_code.to_string(), "4_2_0".to_string());

        assert_eq!(actual_result, Some(true))
    }

    #[test]
    fn test_compile_invalid_code() {
        let actual_result = compile_check("".to_string(), "4_2_0".to_string());

        assert_eq!(actual_result, Some(false))
    }
}