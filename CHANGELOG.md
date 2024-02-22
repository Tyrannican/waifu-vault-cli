# Changelog

An overview of what changes between versions and why

## v0.2.0

* Added the `url` option to the `download` flag in the CLI
    * The `token` is only required for when requesting information about the file
    and not for downloading
    * This simplifies the download logic considerably
* Removed the `token` option for the `download` flag in the CLI
    * See above
* Better logic for determining download locations
* Minor improvements
* Added code documentation for better clarity
