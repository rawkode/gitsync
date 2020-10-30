Feature: Sync a Git Repository

    Example: Nothing has been cloned previously
        Given the local directory does not exist
        When I sync a Git repository
        Then the repository is cloned
