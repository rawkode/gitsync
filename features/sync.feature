Feature: Sync

    GitSync will fetch the latest changes to the repository and if
    possible merge changes into the sync'd branch

    Background:

        Given I have a remote Git repository available

    Example: Remote Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are remote changes
        When I sync
        Then there are changes

    Example: No Remote Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are no remote changes
        When I sync
        Then the sync completes
        And there is no change

    Example: Local Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are local changes
        When I sync
        Then the sync errors
        And there is no change
