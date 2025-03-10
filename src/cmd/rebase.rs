// SPDX-License-Identifier: GPL-2.0-only

//! `stg rebase` implementation.

use std::{fmt::Write, str::FromStr};

use anyhow::{anyhow, Result};
use bstr::ByteSlice;
use clap::{Arg, ArgMatches};

use crate::{
    color::get_color_stdout,
    patchedit::{self, call_editor},
    patchname::PatchName,
    print_info_message,
    revspec::parse_stgit_revision,
    stack::{Stack, StackStateAccess},
    stupid::Stupid,
};

pub(super) const STGIT_COMMAND: super::StGitCommand = super::StGitCommand {
    name: "rebase",
    category: super::CommandCategory::StackManipulation,
    make,
    run,
};

fn make() -> clap::Command {
    clap::Command::new(STGIT_COMMAND.name)
        .about("Move the stack base to another point in history")
        .long_about(
            "Pop all patches from the current stack, move the stack base to the given \
            new base and push the patches back.\n\
            \n\
            Merge conflicts may arise when patches are being pushed-back onto the \
            stack. If this occurs, resolve the conflicts and then continue the rebase \
            with the following sequence:\n\
            \n    \
            stg add --update\n    \
            stg refresh\n    \
            stg goto top-patch\n\
            \n\
            Or to skip the conflicting patch:\n\
            \n    \
            stg undo --hard\n    \
            stg push next-patch..top-patch\n\
            ",
        )
        .arg(
            Arg::new("committish")
                .help("New base commit for the stack")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .required_unless_present("interactive"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .short('i')
                .help("Interactively manipulate patches in editor")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("nopush")
                .long("nopush")
                .short('n')
                .help("Do not push back patches after rebasing")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("merged"),
        )
        .arg(
            Arg::new("merged")
                .long("merged")
                .short('m')
                .help("Check for patches merged upstream")
                .long_help(
                    "Check for patches that may have been merged upstream.\n\
                     \n\
                     When pushing-back patches, each patch is checked to see if its \
                     changes already exist in the new stack base. If a patch's changes \
                     are detected to have been merged, the patch will still exist in \
                     the stack, but become empty after the rebase operation.",
                )
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("autostash")
                .long("autostash")
                .help("Stash changes before the rebase and reapply them after")
                .long_help(
                    "Automatically create a temporary stash before the operation \
                     begins, and apply it after the operation completes. This allows a \
                     rebase to be performed on a dirty work tree. Note however that \
                     the final stash application may result in non-trivial conflicts.",
                )
                .action(clap::ArgAction::SetTrue),
        )
}

fn run(matches: &ArgMatches) -> Result<()> {
    let repo = git2::Repository::open_from_env()?;
    let stack = Stack::from_branch(&repo, None)?;
    let config = repo.config()?;
    let stupid = repo.stupid();
    let branch_name = stack.branch_name.clone();

    let target_commit = if let Some(committish) = crate::argset::get_one_str(matches, "committish")
    {
        parse_stgit_revision(&repo, Some(committish), None)?.peel_to_commit()?
    } else {
        stack.base().clone()
    };

    if stack.is_protected(&config) {
        return Err(anyhow!(
            "This branch is protected. Rebase is not permitted."
        ));
    }

    stack.check_head_top_mismatch()?;
    let clean_result = stupid.statuses(None)?.check_index_and_worktree_clean();

    let autostash = if matches.get_flag("autostash") {
        true
    } else if let Ok(autostash) = config.get_bool("branch.{branch_name}.stgit.autostash") {
        autostash
    } else if let Ok(autostash) = config.get_bool("stgit.autostash") {
        autostash
    } else {
        false
    };

    let using_stash = if autostash && clean_result.is_err() {
        stupid.stash_push()?;
        true
    } else if let Err(e) = clean_result {
        return Err(e);
    } else {
        false
    };

    let applied = stack.applied().to_vec();

    stack
        .setup_transaction()
        .use_index_and_worktree(true)
        .with_output_stream(get_color_stdout(matches))
        .transact(|trans| {
            trans.pop_patches(|pn| applied.contains(pn))?;
            Ok(())
        })
        .execute("rebase (pop)")?;

    let rebase_cmd = config
        .get_string(&format!("branch.{branch_name}.stgit.rebasecmd"))
        .or_else(|_| config.get_string("stgit.rebasecmd"))
        .unwrap_or_else(|_| "git reset --hard".to_string());
    print_info_message(matches, &format!("Rebasing to `{}`", target_commit.id()));
    stupid.user_rebase(&rebase_cmd, target_commit.id())?;

    let stack = Stack::from_branch(&repo, None)?;
    let stack = if stack.is_head_top() {
        stack
    } else {
        // Record a new stack state with updated head since the head moved.
        stack.log_external_mods(Some("rebase"))?
    };

    if matches.get_flag("interactive") {
        interactive_pushback(stack, &repo, &config, matches, &applied)?;
    } else if !matches.get_flag("nopush") {
        stack.check_head_top_mismatch()?;
        let check_merged = matches.get_flag("merged");
        stack
            .setup_transaction()
            .use_index_and_worktree(true)
            .with_output_stream(get_color_stdout(matches))
            .transact(|trans| trans.push_patches(&applied, check_merged))
            .execute("rebase (reapply)")?;
    }

    if using_stash {
        if stupid.stash_pop()? {
            Ok(())
        } else {
            Err(
                crate::stack::Error::CausedConflicts("Stash pop resulted in conflicts".to_string())
                    .into(),
            )
        }
    } else {
        Ok(())
    }
}

const INTERACTIVE_APPLY_LINE: &str = "# --- APPLY_LINE ---";
const INTERACTIVE_HELP_LINES: &str = "\
# Commands:
#
#   k, keep <patch> = do not modify this patch
#   e, edit <patch> = interactively edit this patch
#   s, squash <patch> = squash patch into the previous patch
#   f, fixup <patch> = like \"squash\", but discard this patch's commit message
#   h, hide <patch> = hide patch
#   d, delete <patch> = delete patch
#
# These lines can be reordered; they are executed from top to bottom.
#
# Patches above the APPLY_LINE are applied; other patches are kept unapplied.
";

#[derive(Debug, Clone)]
struct Instruction {
    patchname: PatchName,
    action: Action,
    apply: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Keep,
    Edit,
    Squash,
    Fixup,
    Hide,
    Delete,
}

fn interactive_pushback(
    stack: Stack,
    repo: &git2::Repository,
    config: &git2::Config,
    matches: &ArgMatches,
    previously_applied: &[PatchName],
) -> Result<()> {
    let mut stack = stack;

    if stack.all_patches().next().is_none() {
        return Ok(());
    }

    let filename = ".stgit-rebase-interactive.txt";
    std::fs::write(
        filename,
        make_instructions_template(&stack, previously_applied),
    )?;

    let buf = call_editor(filename, config)?;
    let buf = buf
        .to_str()
        .map_err(|_| anyhow!("`{filename}` is not valid UTF-8"))?;
    let mut instructions = parse_instructions(buf)?;

    validate_instructions(&stack, &instructions)?;

    let mut index: usize = 0;

    while index < instructions.len() {
        let instruction = instructions[index].clone();
        let patchname = &instruction.patchname;

        match instruction.action {
            Action::Keep => {
                index += 1;
            }

            Action::Delete => {
                // Find contiguous delete instructions in order to delete in batches.
                let mut delete_instructions: Vec<Instruction> = Vec::new();
                while index < instructions.len() && instructions[index].action == Action::Delete {
                    delete_instructions.push(instructions.remove(index));
                }
                assert!(!delete_instructions.is_empty());
                assert!(delete_instructions
                    .iter()
                    .all(|inst| inst.action == Action::Delete));
                let to_delete: Vec<&PatchName> = delete_instructions
                    .iter()
                    .map(|inst| &inst.patchname)
                    .collect();
                stack = stack
                    .setup_transaction()
                    .with_output_stream(get_color_stdout(matches))
                    .transact(|trans| {
                        let popped_extra = trans.delete_patches(|pn| to_delete.contains(&pn))?;
                        assert!(popped_extra.is_empty());
                        Ok(())
                    })
                    .execute("delete")?;
            }

            Action::Hide => {
                let mut hide_instructions: Vec<Instruction> = Vec::new();
                while index < instructions.len() && instructions[index].action == Action::Hide {
                    hide_instructions.push(instructions.remove(index));
                }
                assert!(!hide_instructions.is_empty());
                assert!(hide_instructions
                    .iter()
                    .all(|inst| inst.action == Action::Hide));
                let to_hide: Vec<PatchName> = hide_instructions
                    .iter()
                    .map(|inst| inst.patchname.clone())
                    .collect();
                stack = stack
                    .setup_transaction()
                    .with_output_stream(get_color_stdout(matches))
                    .transact(|trans| trans.hide_patches(&to_hide))
                    .execute("hide")?;
            }

            Action::Edit => {
                let dummy_edit_command = clap::Command::new("dummy-edit");
                let dummy_edit_command = patchedit::add_args(dummy_edit_command, false, false);
                let edit_matches = dummy_edit_command
                    .try_get_matches_from(["dummy-edit", "--edit", "--diff"])
                    .expect("dummy command has valid arguments");
                match patchedit::EditBuilder::default()
                    .original_patchname(Some(patchname))
                    .existing_patch_commit(stack.get_patch_commit(patchname))
                    .allow_diff_edit(true)
                    .edit(&stack, repo, &edit_matches)?
                {
                    patchedit::EditOutcome::TemplateSaved(_) => panic!("template save not enabled"),
                    patchedit::EditOutcome::Committed {
                        patchname: new_patchname,
                        commit_id,
                    } => {
                        stack = stack
                            .setup_transaction()
                            .with_output_stream(get_color_stdout(matches))
                            .transact(|trans| {
                                if &new_patchname != patchname {
                                    trans.rename_patch(patchname, &new_patchname)?;
                                }
                                trans.update_patch(&new_patchname, commit_id)?;
                                Ok(())
                            })
                            .execute(&format!("edit: {patchname}"))?;

                        instructions[index] = Instruction {
                            action: Action::Keep,
                            patchname: new_patchname,
                            apply: instruction.apply,
                        };

                        index += 1;
                    }
                }
            }

            Action::Squash | Action::Fixup => {
                let action_str = match instruction.action {
                    Action::Squash => "squash",
                    Action::Fixup => "fixup",
                    _ => panic!("only squash and fixup expected"),
                };

                let squash_patchnames: Vec<PatchName> = if index > 0 {
                    let mut patchnames: Vec<PatchName> =
                        vec![instructions[index - 1].patchname.clone()];
                    while index < instructions.len()
                        && instructions[index].action == instruction.action
                    {
                        let Instruction { patchname, .. } = instructions.remove(index);
                        patchnames.push(patchname);
                    }
                    patchnames
                } else {
                    return Err(anyhow!(
                        "Cannot {action_str} `{patchname}`: no preceding patch"
                    ));
                };

                let target_patchname = &squash_patchnames[0];

                let dummy_squash_command = clap::Command::new("dummy-squash");
                let dummy_squash_command = patchedit::add_args(dummy_squash_command, true, false);
                let squash_matches = match instruction.action {
                    Action::Squash => {
                        dummy_squash_command.try_get_matches_from(["dummy-squash", "--edit"])
                    }
                    Action::Fixup => {
                        let commit = stack.get_patch_commit(target_patchname);
                        let message = commit.message_raw().ok_or_else(|| {
                            anyhow!("Fixup target patch `{target_patchname}` has non-UTF-8 message")
                        })?;
                        dummy_squash_command.try_get_matches_from([
                            "dummy-squash",
                            "--message",
                            message,
                        ])
                    }
                    _ => panic!("only squash and fixup expected"),
                }
                .expect("dummy command has valid arguments");

                stack = stack
                    .setup_transaction()
                    .with_output_stream(get_color_stdout(matches))
                    .transact(|trans| {
                        let new_patchname = super::squash::squash(
                            trans,
                            config,
                            &squash_matches,
                            &squash_patchnames,
                            Some(target_patchname),
                            false,
                        )?;
                        instructions[index - 1] = Instruction {
                            action: Action::Keep,
                            patchname: new_patchname,
                            apply: instruction.apply,
                        };
                        Ok(())
                    })
                    .execute("squash")?;
            }
        }
    }

    let to_push: Vec<PatchName> = instructions
        .iter()
        .filter_map(|inst| {
            if inst.apply {
                Some(inst.patchname.clone())
            } else {
                None
            }
        })
        .collect();
    let check_merged = matches.get_flag("merged");

    stack.check_head_top_mismatch()?;
    stack
        .setup_transaction()
        .use_index_and_worktree(true)
        .with_output_stream(get_color_stdout(matches))
        .transact(|trans| trans.push_patches(&to_push, check_merged))
        .execute("rebase (reapply)")?;

    Ok(())
}

fn make_instructions_template(stack: &Stack, previously_applied: &[PatchName]) -> String {
    let name_width = stack.all_patches().map(PatchName::len).max().unwrap();
    let mut template = String::with_capacity(4096);
    let mut found_apply_boundary = false;
    for patchname in stack.all_patches() {
        if !found_apply_boundary && !previously_applied.contains(patchname) {
            writeln!(template, "{INTERACTIVE_APPLY_LINE}").unwrap();
            found_apply_boundary = true;
        }
        let commit = stack.get_patch_commit(patchname);
        let subject = commit.summary().unwrap_or_default();
        writeln!(template, "keep {patchname:name_width$} # {subject}").unwrap();
    }
    if !found_apply_boundary {
        writeln!(template, "{INTERACTIVE_APPLY_LINE}").unwrap();
    }
    template.push_str(INTERACTIVE_HELP_LINES);
    template
}

fn parse_instructions(buf: &str) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let mut apply = true;

    for line in buf.lines() {
        let line = line.trim();

        if line.contains(INTERACTIVE_APPLY_LINE) {
            if line == INTERACTIVE_APPLY_LINE {
                apply = false;
            } else {
                return Err(anyhow!("Bad APPLY_LINE: `{line}`"));
            }
        }

        let instruction_str = if let Some((instruction_str, _comment)) = line.split_once('#') {
            instruction_str
        } else {
            line
        }
        .trim();

        if instruction_str.is_empty() {
            continue;
        }

        if let Some((action_str, patchname_str)) =
            instruction_str.split_once(|c: char| c.is_ascii_whitespace())
        {
            let action = match action_str {
                "k" | "keep" => Action::Keep,
                "e" | "edit" => Action::Edit,
                "s" | "squash" => Action::Squash,
                "f" | "fix" | "fixup" => Action::Fixup,
                "h" | "hide" => Action::Hide,
                "d" | "delete" => Action::Delete,
                _ => return Err(anyhow!("Unknown instruction action `{action_str}`")),
            };

            let patchname = PatchName::from_str(patchname_str)?;

            instructions.push(Instruction {
                patchname,
                action,
                apply,
            });
        } else {
            return Err(anyhow!("Bad instruction line: `{line}`"));
        }
    }
    Ok(instructions)
}

fn validate_instructions(stack: &Stack, instructions: &[Instruction]) -> Result<()> {
    let mut seen_patchnames: Vec<&PatchName> = Vec::new();
    for instruction in instructions {
        let patchname = &instruction.patchname;
        if !stack.has_patch(patchname) {
            return Err(anyhow!("Unknown patch name `{patchname}`"));
        } else if seen_patchnames.contains(&patchname) {
            return Err(anyhow!("Duplicated patch name `{patchname}`"));
        } else {
            seen_patchnames.push(patchname);
        }
    }
    Ok(())
}
