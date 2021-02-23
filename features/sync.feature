Feature: Sync

    GitSync will fetch the latest changes to the repository and if
    possible merge changes into the sync'd branch

    Example: No Remote Changes

        Given I have a Git repository in a directory called "gitsync"
        And there are no remote changes
        When I sync
        Then there is no change
