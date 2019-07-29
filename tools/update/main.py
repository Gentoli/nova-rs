import argparse
import functools
import re
import typing
import sys

##########
# Config #
##########

# Regex and files for changing the rust toolchain
toolchain_version_regex = re.compile(r"(stable|beta|nightly)-\d{4}-\d{2}-\d{2}", re.UNICODE)
toolchain_files = [
    ".appveyor.yml",
    ".travis.yml",
    "README.md",
    "rust-toolchain",
]


# Regex for updating version number
version_simple_regex = re.compile(r"""(\d+).(\d+).(\d+)(-\w+)?""", re.MULTILINE | re.UNICODE)
version_cargo_regex = re.compile(r"""^version\s*=\s*"(\d+).(\d+).(\d+)(-[^"]+)?"\s*$""", re.MULTILINE | re.UNICODE)


#########
# Utils #
#########


def replace_in_file(file: str, pattern: typing.Pattern[str], replacement: str) -> None:
    """
    Helper to do regex replacement inside an entire file.

    :param file:        File to replace inside
    :param pattern:     Pre-compiled pattern to match with
    :param replacement: Replacement string passed to `re.sub`
    :return:            None
    """
    try:
        with open(file, "r") as f:
            contents = f.read()
    except FileNotFoundError:
        print("File {} not found.".format(file), file=sys.stderr)
        exit(1)

    replaced = pattern.sub(replacement, contents)

    with open(file, "w") as f:
        # In case this doesn't truncate for some reason
        f.truncate()
        f.write(replaced)


###########
# Version #
###########


def validate_version(version: str) -> str:
    """
    Verify that a string matches a valid Nova version number.

    ex: 0.2.3-superbeta
    ex: 1.9.2

    :param version:                     Version string to match against
    :return:                            Original String
    :raises argparse.ArgumentTypeError: When invalid
    """
    if version_simple_regex.match(version) is None:
        msg = "\"{}\" is not a valid Nova version. ex: 0.2.3-superbeta".format(version)
        raise argparse.ArgumentTypeError(msg)
    return version


def replace_version(new_version: str) -> None:
    """
    Replaces Nova version in all files that need to updated.

    Currently:
    - Cargo.toml

    :param new_version: Replacement version. unvalidated.
    :return:            None
    """
    replace_in_file("Cargo.toml", version_cargo_regex, """version = \"{}\"""".format(new_version))
    print("Updated {}".format(new_version))
    print("Please run `cargo update -p nova-rs` to update Cargo.lock")


##################
# Rust Toolchain #
##################


def validate_toolchain_version(version: str) -> str:
    """
    Verify that a string matches a platform agnostic rust toolchain version.

    ex: stable-2019-07-04
    ex: nightly-2019-07-23

    :param version:                     Version string to match against
    :return:                            Original String.
    :raises argparse.ArgumentTypeError: When invalid
    """
    if toolchain_version_regex.match(version) is None:
        msg = "\"{}\" is not a valid toolchain version. ex: nightly-2019-07-23".format(version)
        raise argparse.ArgumentTypeError(msg)
    return version


def replace_toolchain_version(new_toolchain: str) -> None:
    """
    Replaces rust toolchain version in all files that need to updated.

    Currently:
    - .appveyor.yml
    - .travis.yml
    - README.md
    - rust-toolchain

    :param new_toolchain: Replacement toolchain. unvalidated.
    :return:              None
    """
    for file in toolchain_files:
        replace_in_file(file, toolchain_version_regex, new_toolchain)
        print("Updated {}".format(file))


########
# Main #
########


commit_warning = "Please commit changes made by this tool in their own commit and PR."


def main():
    # noinspection PyTypeChecker
    parser = argparse.ArgumentParser(description="nova-rs version updater\n",
                                     epilog=commit_warning,
                                     formatter_class=functools.partial(argparse.HelpFormatter,
                                                                       max_help_position=40,
                                                                       width=120))
    subparsers = parser.add_subparsers(title="commands", dest="command", required=True)

    # Rust Toolchain
    toolchain_p = subparsers.add_parser("toolchain",
                                        help="update default rust toolchain version",
                                        epilog=commit_warning)
    toolchain_p.add_argument("toolchain_version",
                             type=validate_toolchain_version,
                             help="ex: nightly-2019-07-02")

    # Nova Version
    toolchain_p = subparsers.add_parser("version",
                                        help="update default rust toolchain version",
                                        epilog=commit_warning)
    toolchain_p.add_argument("nova_version",
                             type=validate_version,
                             help="ex: 0.5.4-superbeta")

    parsed = parser.parse_args()

    if parsed.command == "version":
        replace_version(parsed.nova_version)
    elif parsed.command == "toolchain":
        replace_toolchain_version(parsed.toolchain_version)

    print(commit_warning)
