# NAME

wpscan-analyze -- Analyzes wpscan json output and checks for vulnerabilities.


# SYNOPSIS

wpscan-analyze [*options*] *MODULE*

wpscan-analyze --help

wpscan-analyze --version


# DESCRIPTION

wpscan-analyze is a CLI tool that analyzes wpscan json output and checks for vulnerabilities.

The project home page currently is *https://github.com/lukaspustina/wpscan-analyze*.


# COMMON OPTIONS

-f, --wpscan *wpscan*
: wpscan JSON file

--output-detail *output_detail*
: Select output detail level for human output; all or nok (not ok) [default: nok]  [possible values: nok, all]. Please mind that results for WordPress and its main theme are always displayed.

-o, --output *output_format*
: Select output format [default: human]  [possible values: human, json, none]

-v, --verbose
: Verbose mode (-v, -vv, -vvv, etc.)

--help
: Prints help information


# LESS COMMON OPTIONS

--no-color
: Turns off colorful output. Helpful for non-tty usage.

-s, --silent
: Silencium; use this for json output.

-V, --version
: Prints version information.


# EXIT STATUS

If wpscan-analyze does not find any issue, then the exit status of the program is 0. The exit states 11, 12, and 13 signal that wpscan-analyze has encountered a vulnerability, an outdated element, or a processing failure, respectively. Exit status 14 signals that the analysis is inconclusive, because elements with unknown versions have been found. It is up to the user to assess what this means in his specific situation. Any other exit status signals an error.


# COPYRIGHT AND LICENSE

Copyright (c) 2019 Lukas Pustina. Licensed under the MIT License. See *https://github.com/lukaspustina/wpscan-analyze/blob/master/LICENSE* for details.

