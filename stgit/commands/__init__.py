import os

from stgit.compat import text

__copyright__ = """
Copyright (C) 2005, Catalin Marinas <catalin.marinas@gmail.com>
Copyright (C) 2008, Karl Hasselström <kha@treskal.com>

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License version 2 as
published by the Free Software Foundation.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, see http://www.gnu.org/licenses/.
"""


def get_command(mod_name):
    """Import and return the given command module."""
    return __import__(__name__ + '.' + mod_name, globals(), locals(), ['*'])


_kinds = [
    ('repo', 'Repository commands'),
    ('stack', 'Stack (branch) commands'),
    ('patch', 'Patch commands'),
    ('wc', 'Index/worktree commands'),
    ('alias', 'Alias commands'),
]
_kind_order = [kind for kind, desc in _kinds]
_kinds = dict(_kinds)


def _find_commands():
    for p in __path__:
        for fn in os.listdir(p):
            mod_name, ext = os.path.splitext(fn)
            if mod_name.startswith('_') or ext != '.py':
                continue
            mod = get_command(mod_name)
            if not hasattr(mod, 'usage'):
                continue
            yield mod_name, mod


def get_commands(allow_cached=True):
    """Return list of tuples of command name, module name, command type, and
    one-line command help."""
    if allow_cached:
        try:
            from stgit.commands.cmdlist import command_list

            return command_list
        except ImportError:
            # cmdlist.py doesn't exist, so do it the expensive way.
            pass
    return sorted(
        (
            text(getattr(mod, 'name', mod_name)),
            text(mod_name),
            _kinds[mod.kind],
            mod.help,
        )
        for mod_name, mod in _find_commands()
    )


def py_commands(commands, f):
    lines = [
        '# This file is autogenerated.',
        '',
        'command_list = [',
    ]
    for cmd_name, mod_name, kind, help in commands:
        lines.extend(
            [
                "    (",
                "        '%s'," % cmd_name,
                "        '%s'," % mod_name,
                "        '%s'," % kind,
                "        '''%s'''," % help,
                "    ),",
            ]
        )
    lines.append(']')
    lines.append('')
    f.write('\n'.join(lines))


def _command_list(commands):
    kinds = {}
    for cmd, mod, kind, help in commands:
        kinds.setdefault(kind, {})[cmd] = help
    for kind in _kind_order:
        kind = _kinds[kind]
        try:
            yield kind, sorted(kinds[kind].items())
        except KeyError:
            pass


def pretty_command_list(commands, f):
    cmd_len = max(len(cmd) for cmd, _, _, _ in commands)
    sep = ''
    for kind, cmds in _command_list(commands):
        f.write(sep)
        sep = '\n'
        f.write('%s:\n' % kind)
        for cmd, help in cmds:
            f.write('  %*s  %s\n' % (-cmd_len, cmd, help))


def _write_underlined(s, u, f):
    f.write(s + '\n')
    f.write(u * len(s) + '\n')


def asciidoc_command_list(commands, f):
    for kind, cmds in _command_list(commands):
        _write_underlined(kind, '~', f)
        f.write('\n')
        for cmd, help in cmds:
            f.write('linkstg:%s[]::\n' % cmd)
            f.write('    %s\n' % help)
        f.write('\n')
