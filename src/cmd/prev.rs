// SPDX-License-Identifier: GPL-2.0-only

//! `stg prev` implementation.

use std::io::Write;

use anyhow::{anyhow, Result};
use termcolor::WriteColor;

use crate::{
    argset,
    stack::{Stack, StackStateAccess},
};

pub(super) const STGIT_COMMAND: super::StGitCommand = super::StGitCommand {
    name: "prev",
    category: super::CommandCategory::StackInspection,
    make,
    run,
};

fn make() -> clap::Command {
    clap::Command::new(STGIT_COMMAND.name)
        .about("Print the name of the previous patch")
        .long_about(
            "Print the name of the previous patch.\n\
             \n\
             The previous patch is the applied patch preceding the current, \
             topmost patch. An error message will be printed if not enough \
             patches are applied.",
        )
        .arg(argset::branch_arg())
}

fn run(matches: &clap::ArgMatches) -> Result<()> {
    let repo = git2::Repository::open_from_env()?;
    let opt_branch = argset::get_one_str(matches, "branch");
    let stack = Stack::from_branch(&repo, opt_branch)?;

    if let Some(patchname) = stack.applied().iter().nth_back(1) {
        let mut stdout = crate::color::get_color_stdout(matches);
        let mut color_spec = termcolor::ColorSpec::new();
        color_spec.set_bold(true);
        stdout.set_color(&color_spec)?;
        write!(stdout, "{patchname}")?;
        color_spec.clear();
        stdout.set_color(&color_spec)?;
        writeln!(stdout)?;
        Ok(())
    } else {
        Err(anyhow!("Not enough patches applied"))
    }
}
