# Code Review Bot (cr-bot)

The `cr-bot` is a command line application developed in Rust.
It leverages the GPT4 model to review code.

## Installation

You can compile the bot from source using the following command:

```shell
cargo build --release
```

The resulting binary will be located in the `target/release` directory.

## Requirements

This program relies on two environment variables to function correctly. Set the following before running the application:

1. `OPENAI_API_KEY`: This is your OpenAI API Key. It allows the program to access the OpenAI API for processing and generating the data.

2. `GH_PR_TOKEN`: This is a GitHub API token with 'repo' scope. It's used to fetch details from pull requests during the reviews. Note that while this is needed for reviewing private repositories, it's not required for public repositories.

Ensure both of these environment variables are properly set in your shell or the environment where this program will run.

## Usage

The bot can be invoked in two ways:

### Review a GitHub PR

To review a PR use the following format:

```shell
cr-bot <owner> <repo> <pr_number>
```

e.g.

```shell
cr-bot nihilok cr-bot 9
```

### Review Local Changes

To review changes on your current branch against `main`, use the `--local` option:

```shell
cr-bot --local
```

For additional help:

```shell
cr-bot --help
```

## Configuration

You may edit the system message for the bot before compile time by modifying the `src/system-message.txt` file.

## License

[MIT](./LICENSE)