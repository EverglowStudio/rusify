use std::ops::Not;
use std::process::{Command, Stdio};

use crate::common::models::Config;
use crate::Result;
use indicatif::MultiProgress;

use super::{CommandInfo, CommandSpinner, MainSpinner, OptionalMultiProgress, Ticking};

pub fn run_step<T, E, S>(config: &Config, title: S, execute: E) -> Result<T>
where
    E: FnOnce() -> Result<T>,
    S: ToString,
{
    let spinner = config
        .silent
        .not()
        .then_some(MainSpinner::with_message(title.to_string()));

    spinner.start();

    let result = execute();

    match result {
        Ok(_) => spinner.finish(),
        Err(_) => spinner.fail(),
    }

    result
}

pub fn run_step_with_commands<S>(config: &Config, title: S, commands: &mut [Command]) -> Result<()>
where
    S: ToString,
{
    let multi = config.silent.not().then(MultiProgress::new);
    let spinner = config
        .silent
        .not()
        .then_some(MainSpinner::with_message(title.to_string()));
    multi.add(&spinner);
    spinner.start();

    for command in commands {
        let step = config
            .silent
            .not()
            .then(|| CommandSpinner::with_command(command));
        multi.add(&step);
        step.start();

        let output = command
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute command: {}", command.info()));

        if !output.status.success() {
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            // Print full output when error occurs
            if !stderr_output.is_empty() {
                eprintln!("\n==== STDERR OUTPUT ====\n{}", stderr_output);
            }

            step.fail();
            spinner.fail();
            return Err(anyhow::anyhow!(
                "{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        step.finish();
    }

    spinner.finish();
    Ok(())
}
