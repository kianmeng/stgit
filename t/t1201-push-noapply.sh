#!/bin/sh

test_description='Exercise pushing with the --noapply option'

. ./test-lib.sh

test_expect_success 'Setup patches' '
    test_commit_bulk --message="a%s" --filename=a.txt --contents="line %s" 3 &&
    test_commit_bulk --message="b%s" --filename=b.txt --contents="line %s" 3 &&
    stg init &&
    stg uncommit -n 6 &&
    stg pop -a &&
    test "$(echo $(stg series --unapplied --noprefix))" = "a1 a2 a3 b1 b2 b3"
'

test_expect_success 'Check --noapply with --all' '
    general_error stg push --noapply --all 2>err &&
    grep -e "error: The argument .--noapply. cannot be used with .--all." err
'

test_expect_success 'Check --noapply with --number' '
    general_error stg push --noapply -n 3 2>err &&
    grep -e "error: The argument .--noapply. cannot be used with .--number <n>." err
'

test_expect_success 'Check --noapply without patch names' '
    general_error stg push --noapply 2>err &&
    grep -e "error: The following required arguments were not provided:" err
'

test_expect_success 'Check --noapply with --set-tree' '
    general_error stg push --noapply --set-tree b1 b2 b3 2>err &&
    grep -e "error: The argument .--noapply. cannot be used with .--set-tree." err
'

test_expect_success 'Check --noapply with --merged' '
    general_error stg push --noapply -m b1 b2 b3 2>err &&
    grep -e "error: The argument .--noapply. cannot be used with .--merged." err
'

test_expect_success 'Reorder patches b1 b2 b3' '
    stg push --noapply b1 b2 b3 &&
    test "$(echo $(stg series --unapplied --noprefix))" = "b1 b2 b3 a1 a2 a3"
'

test_expect_success 'Push reorded patches b1 b2 b3' '
    stg push -n 3 &&
    test "$(echo $(stg series --applied --noprefix))" = "b1 b2 b3" &&
    test "$(echo $(stg series --unapplied --noprefix))" = "a1 a2 a3"
'

test_expect_success 'Attempt push --noapply on applied patch' '
    command_error stg push --noapply b1 2>err &&
    grep -e "error: Patch \`b1\` is already applied" err
'

test_expect_success 'Reorder patches to cause a latent conflict' '
    stg push --noapply a1 a3 &&
    test "$(echo $(stg series --applied --noprefix))" = "b1 b2 b3" &&
    test "$(echo $(stg series --unapplied --noprefix))" = "a1 a3 a2"
'

test_expect_success 'Observe latent conflict with regular push' '
    stg push &&
    test "$(echo $(stg series --applied --noprefix))" = "b1 b2 b3 a1" &&
    test "$(echo $(stg series --unapplied --noprefix))" = "a3 a2" &&
    conflict stg push &&
    echo "line 3" > a.txt &&
    stg add a.txt &&
    stg refresh &&
    test "$(echo $(stg series --applied --noprefix))" = "b1 b2 b3 a1 a3" &&
    test "$(echo $(stg series --unapplied --noprefix))" = "a2"
'

test_expect_success 'Push allowed with dirty worktree' '
    stg pop -n 2 &&
    echo "foobar" > b.txt &&
    test_when_finished git checkout b.txt &&
    test "$(stg status b.txt)" = " M b.txt" &&
    stg push --noapply a1 a2 a3 &&
    test "$(echo $(stg series --applied --noprefix))" = "b1 b2 b3" &&
    test "$(echo $(stg series --unapplied --noprefix))" = "a1 a2 a3"
'

test_done
