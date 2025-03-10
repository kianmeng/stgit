# Stacked Git

Stacked Git, **StGit** for short, is an application for managing Git
commits as a stack of patches.

With a *patch stack* workflow, multiple patches can be developed
concurrently and efficiently, with each patch focused on a single
concern, resulting in both a clean Git commit history and improved
productivity.

For a complete introduction to StGit, see the [Stacked Git
homepage](https://stacked-git.github.io).

## Getting started

To get a feel for how StGit works, see this brief [example of StGit in
action][example]. Or check out the [in-depth tutorial][tutorial].

[example]: https://stacked-git.github.io/guides/usage-example
[tutorial]: https://stacked-git.github.io/guides/tutorial

StGit also has a complete set of [man pages][man] describing the
[`stg`][stg] command line tool and each of its subcommands.

[man]: https://stacked-git.github.io/man
[stg]: https://stacked-git.github.io/man/stg

## Installation

See [CHANGELOG.md](CHANGELOG.md) to see what has changed in the latest
StGit release.

### Dependencies

StGit is implemented in Rust using a number of third-party, open source
crates. StGit statically links with its pure-Rust dependencies, but
dynamically links to libc and other non-Rust libraries when they are
available at build-time. Dynamic link dependencies include these
libraries along with their transient link dependencies:

- libgit2
- libcurl (optional)
- libbz2 (optional)

StGit works within the context of a Git repository and performs many
operations by running subordinate `git` commands.
[Git](https://git-scm.com) 2.2.0 or newer is required.

### Package Repositories

Recent versions of StGit are available via many package repositories
such as [HomeBrew](https://formulae.brew.sh/formula/stgit) and for many
Linux distributions including:
[Alpine](https://pkgs.alpinelinux.org/packages?name=stgit),
[Arch](https://aur.archlinux.org/packages/stgit),
[Fedora](https://src.fedoraproject.org/rpms/stgit),
[Nix](https://nixos.org/nixos/packages.html?attr=gitAndTools.stgit) and
[Ubuntu](https://packages.ubuntu.com/source/focal/stgit).

More details about StGit packages availability for various operating
systems can be [found on repology][repology].

[repology]: https://repology.org/project/stgit/versions

### Source Installation

StGit may also be installed from source. Download the [latest
release][latest] or clone from the [StGit repository on GitHub][repo].

[latest]: https://github.com/stacked-git/stgit/releases/latest
[repo]: https://github.com/stacked-git/stgit

To install the `stg` executable from source, choose a `prefix` and run:

```shellsession
$ make prefix=$HOME/.local install
```

For more information about installation, see [INSTALL.md](INSTALL.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for a full guide to contributing
to StGit.

## Maintainers

StGit is maintained by Catalin Marinas and Peter Grayson.

For a complete list of StGit's authors, see [AUTHORS.md](AUTHORS.md).
