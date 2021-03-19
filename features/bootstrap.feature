Feature: Bootstrap

    GitSync expects to clone a directory to a new directory
    but we have rules for some existing dirs

    Background:

        Given I have a remote Git repository available

    Example: No local directory

        Given I have no directory called "gitsync"
        When I bootstrap
        Then the repository is cloned
        And the bootstrap completes

    Example: Local directory isn't a Git repository

        Given I have a directory called "gitsync"
        And it contains a file called "random.txt"
        When I bootstrap
        Then the directory is left untouched
        And the bootstrap errors because "local dir isn't git repository"

    Rule: If we have a local clone the origin remote must be correct

        Example: Local clone has incorrect url for origin remote

            Given I have a Git repository in a directory called "gitsync"
            But it has a remote called "origin" that points to "https://github.com/rawkode/gitsync"
            When I bootstrap
            Then the directory is left untouched
            And the bootstrap errors because "incorrect remote"

        Example: Local clone has no remote called "origin"

            Given I have a Git repository in a directory called "gitsync"
            But it has no remote called "origin"
            When I bootstrap
            Then the directory is left untouched
            And the bootstrap errors because "incorrect remote"


        Example: Local clone has the correct origin remote

            Given I have a Git repository in a directory called "gitsync"
            And it has a correctly configured remote called "origin"
            When I bootstrap
            Then the bootstrap completes
