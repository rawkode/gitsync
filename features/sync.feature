Feature: Sync

    GitSync will fetch the latest changes to the repository and if
    possible merge changes into the sync'd branch

    Background:

        Given I have a remote Git repository available

    Example: Remote Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are remote changes
        When I sync
        Then the sync reports changes
        And there are changes

    Example: No Remote Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are no remote changes
        When I sync
        Then the sync completes
        And there is no change
        And the sync reports no changes
        And head_oid matches HEAD

    Example: Local Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are local changes
        When I sync
        Then the sync errors
        And there is no change

    Example: Non-default branch changes

        Given the remote has a branch called "config"
        And I have no directory called "gitsync"
        When I bootstrap branch "config"
        Then the bootstrap completes
        And the checked out branch is "config"
        Given there are remote changes on branch "config"
        When I sync branch "config"
        Then the sync reports changes
        And there are changes
        And the checked out branch is "config"
        When I sync branch "config"
        Then the sync reports no changes
