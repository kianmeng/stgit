#!/bin/sh
#
# Copyright (c) 2006 Catalin Marinas
#

test_description='Test the sync command.'

. ./test-lib.sh

test_expect_success \
    'Initialize the StGit repository' \
    '
    stg init
    '

test_expect_success \
    'Create some patches' \
    '
    stg new p1 -m p1 &&
    echo foo1 > foo1.txt &&
    stg add foo1.txt &&
    stg refresh &&
    stg new p2 -m p2 &&
    echo foo2 > foo2.txt &&
    stg add foo2.txt &&
    stg refresh &&
    stg new p3 -m p3 &&
    echo foo3 > foo3.txt &&
    stg add foo3.txt &&
    stg refresh &&
    stg export &&
    stg pop &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "p3" ]
    '

test_expect_success \
    'Create a branch with empty patches' \
    '
    stg branch -c foo {base} &&
    stg new p1 -m p1 &&
    stg new p2 -m p2 &&
    stg new p3 -m p3 &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ]
    '

test_expect_success \
    'Attempt sync with current branch' \
    '
    command_error stg sync -B foo 2>err &&
    grep -e "Cannot synchroni[sz]e with the current branch" err
    '

test_expect_success \
    'Attempt sync without remote branch or series' \
    '
    general_error stg sync -a 2>err &&
    grep -e "The following required arguments were not provided" err
    '

test_expect_success \
    'Attempt apply top patch without any applied' \
    '
    test_when_finished "stg goto p3" &&
    stg pop -a &&
    command_error stg sync -B master 2>err &&
    grep -e "[Nn]o patches applied" err
    '

test_expect_success \
    'Attempt to sync patch not in ref branch' \
    '
    stg new -m p4 &&
    test_when_finished "stg delete p4" &&
    command_error stg sync -B master 2>err &&
    grep -e "No common patches to synchronize" err
    '

test_expect_success \
    'Synchronise second patch with the master branch' \
    '
    stg sync -B master p2 &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ] &&
    test "$(cat foo2.txt)" = "foo2"
    '

test_expect_success \
    'Synchronise the first two patches with the master branch' \
    '
    stg sync -B master -a &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ] &&
    test "$(cat foo1.txt)" = "foo1" &&
    test "$(cat foo2.txt)" = "foo2"
    '

test_expect_success \
    'Synchronise all the patches with the exported series' \
    '
    stg sync -s patches-master/series -a &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ] &&
    test "$(cat foo1.txt)" = "foo1" &&
    test "$(cat foo2.txt)" = "foo2" &&
    test "$(cat foo3.txt)" = "foo3"
    '

test_expect_success \
    'Modify the master patches' \
    '
    stg branch master &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "p3" ] &&
    stg goto p1 &&
    echo bar1 >> foo1.txt &&
    stg refresh &&
    stg goto p2 &&
    echo bar2 > bar2.txt &&
    stg add bar2.txt &&
    stg refresh &&
    stg goto p3 &&
    echo bar3 >> foo3.txt &&
    stg refresh &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ] &&
    stg export &&
    stg branch foo
    '

test_expect_success \
    'Synchronise second patch with the master branch' \
    '
    stg goto p1 &&
    stg sync -B master p2 &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "p3" ] &&
    test "$(cat bar2.txt)" = "bar2"
    '

test_expect_success \
    'Synchronise the first two patches with the master branch (to fail)' \
    '
    conflict stg sync -B master -a
    '

test_expect_success \
    'Restore the stack status after the failed sync' \
    '
    [ "$(echo $(stg series --applied --noprefix))" = "p1" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "p2 p3" ] &&
    stg add --update &&
    stg refresh &&
    stg goto p3
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ]
    '

test_expect_success \
    'Synchronise the third patch with the exported series (to fail)' \
    '
    conflict stg sync -s patches-master/series p3
    '

test_expect_success \
    'Restore the stack status after the failed sync' \
    '
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ] &&
    stg add --update &&
    stg refresh &&
    [ "$(echo $(stg series --applied --noprefix))" = "p1 p2 p3" ] &&
    [ "$(echo $(stg series --unapplied --noprefix))" = "" ]
    '

test_done
