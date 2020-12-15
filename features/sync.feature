@NotImplemented
Feature: Bootstrap Git Repository

    GitSync expects to clone a directory to a new directory
    but we have rules for some existing dirs

    @CLI
    Example: No local directory

        Given I have no directory called "gitsync"
        When I sync the "https://gitlab.com/rawkode/gitsync" repository
        Then the repository is cloned into the "gitsync" directory
        And the bootstrapping completes

    @CLI
    Example: Local directory isn't a Git repository

        Given I have a directory called "gitsync"
        And it is not a Git repository
        When I sync the "https://gitlab.com/rawkode/gitsync" repository
        Then the directory is left untouched
        And the bootstrapping errors

    Rule: If we have a local clone the origin remote must be correct

        Example: Local clone has incorrect url for origin remote

            Given I have local clone called "gitsync"
            But it has a remote called "origin" that points to "https://gitlab.com/rawkode/dotfiles"
            When I sync the "https://gitlab.com/rawkode/gitsync" repository
            Then the directory is left untouched
            And the bootstrapping errors

        Example: Local clone has no remote called "origin"

            Given I have local clone called "gitsync"
            But it has no remote called "origin"
            When I sync the "https://gitlab.com/rawkode/gitsync" repository
            Then the directory is left untouched
            And the bootstrapping errors


        Example: Local clone has the correct origin remote

            Given I have local clone called "gitsync"
            And it has a remote called "origin" that points to "https://gitlab.com/rawkode/gitsync"
            When I sync the "https://gitlab.com/rawkode/gitsync" repository
            Then the bootstrapping completes
