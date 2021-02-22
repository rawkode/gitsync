Feature: Sync

    GitSync will fetch the latest changes to the repository and if
    possible merge changes into the sync'd branch

    Example: No Remote Changes

        Given I have a local clone
        And there are no remote changes
        When I fetch from the remote
        Then there are no changes to merge
