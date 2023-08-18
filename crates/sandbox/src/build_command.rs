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

use crate::docker_command;
use std::{
    path::Path,
    time::Duration, fmt::format,
};
use tokio::process::Command;

const DOCKER_PROCESS_TIMEOUT_SOFT: Duration = Duration::from_secs(20);

const DOCKER_CONTAINER_NAME: &str = "sankarboro/ink-compiler";

const DOCKER_OUTPUT: &str = "/playground-result";

pub fn build_compile_command(input_file: &Path, output_dir: &Path, version: &str) -> Command {
    let mut cmd = build_docker_command(input_file, Some(output_dir), version);

    let execution_cmd = build_execution_command(version);

    cmd.arg(DOCKER_CONTAINER_NAME).args(&execution_cmd);

    log::debug!("Compilation command is {:?}", cmd);

    cmd
}

pub fn build_testing_command(input_file: &Path, version: &str) -> Command {
    let mut cmd = build_docker_command(input_file, None, version);

    let execution_cmd = testing_execution_command();

    cmd.arg(DOCKER_CONTAINER_NAME).args(&execution_cmd);

    log::debug!("Testing command is {:?}", cmd);

    cmd
}

pub fn build_formatting_command(input_file: &Path, version: &str) -> Command {
    let mut cmd = build_docker_command(input_file, None, version);

    let execution_cmd = formatting_execution_command();

    cmd.arg(DOCKER_CONTAINER_NAME).args(&execution_cmd);

    log::debug!("Formatting command is {:?}", cmd);

    cmd
}

fn build_docker_command(input_file: &Path, output_dir: Option<&Path>, version: &str) -> Command {
    let file_name = "lib.rs";
    let work_dir = format!("/app/contract_v{}/", version);
    let mut mount_input_file = input_file.as_os_str().to_os_string();
    mount_input_file.push(":");
    mount_input_file.push(&work_dir);
    mount_input_file.push(file_name);

    let mut cmd = build_basic_secure_docker_command(&work_dir);

    cmd.arg("--volume").arg(&mount_input_file);

    if let Some(output_dir) = output_dir {
        let mut mount_output_dir = output_dir.as_os_str().to_os_string();
        mount_output_dir.push(":");
        mount_output_dir.push(DOCKER_OUTPUT);

        cmd.arg("--volume").arg(&mount_output_dir);
    }

    cmd
}

fn build_basic_secure_docker_command(work_dir: &str) -> Command {
    let mut cmd = docker_command!(
        "run",
        "--detach",
        "--cap-drop=ALL",
        // Needed to allow overwriting the file
        "--cap-add=DAC_OVERRIDE",
        "--security-opt=no-new-privileges",
        "--workdir",
        work_dir,
        "--net",
        "none",
        "--memory",
        "1024m",
        "--memory-swap",
        "1200m",
        "--env",
        format!(
            "PLAYGROUND_TIMEOUT={}",
            DOCKER_PROCESS_TIMEOUT_SOFT.as_secs()
        ),
    );

    if cfg!(feature = "fork-bomb-prevention") {
        cmd.args(["--pids-limit", "512"]);
    }

    cmd.kill_on_drop(true);

    cmd
}

fn build_execution_command(version: &str) -> Vec<String> {
    // let target_dir = "/target/ink";

    let clean_cmd = format!(
        "rm -rf /app/contract_v{}/target/ink/contract_v{}.* /app/contract_v{}/target/ink/metadata.json",
        version, version, version
    );
    let build_cmd = format!("cd /app/contract_v{} && cargo contract build --offline 2>&1", version);
    let move_cmd = format!("mv /app/contract_v{}/target/ink/contract_v{}.contract {}", version, version, DOCKER_OUTPUT);

    let command = format!("{} && {} && {}", clean_cmd, build_cmd, move_cmd);

    let cmd = vec!["/bin/bash".to_string(), "-c".to_string(), command];

    cmd
}

fn testing_execution_command() -> Vec<String> {
    let test_cmd = "cargo test 2>&1".to_string();

    let cmd = vec!["/bin/bash".to_string(), "-c".to_string(), test_cmd];

    cmd
}

fn formatting_execution_command() -> Vec<String> {
    let format_cmd = "cargo +nightly fmt && cat lib.rs 2>&1".to_string();

    let cmd = vec!["/bin/bash".to_string(), "-c".to_string(), format_cmd];

    cmd
}