# Contributing

We really appreciate and value contributions.
Make sure you read those guidelines before starting to make sure your contributions are aligned with project's goals.

## Contribution guidelines

You should always include tests and documentation (e.g. comments) where needed.

To guarantee smooth cooperation and limit redundancies, follow these steps to contribute to the repo:
- ask for assignment on an issue by commenting on it 🙋‍♂️. 
    - It is strongly recommended to indicate the ideas you have to solve the issue (technical conception, architectural choices, etc.) to help maintainers understand your implementation.

- wait for assignment by maintainers. 
    - you can ping maintainers on [discord](https://discord.gg/7RqGMYKT).

- open a pull request or draft pull request and ask for review 💁

- wait for approval and meeeerge! ✅ 🔥

## Creating Pull Requests (PRs)

As a contributor, you are expected to fork this repository, work on your own fork and then submit pull requests.

The pull requests will be reviewed and eventually merged into the main repo. See ["Fork-a-Repo"](https://help.github.com/articles/fork-a-repo/) for methodology.

## Develop and test locally

In order to setup a local environment, please follow the instructions in the [README](./README.md).

## Rust and Cairo-Foundry good practices

-   Making use of [clippy](https://github.com/rust-lang/rust-clippy):

    > A collection of lints to catch common mistakes and improve your Rust code.

    -   Update your Rust version: run `rustup update`
    -   Install clippy: run `rustup component add clippy`
    -   Run clippy and receive some tips: run `cargo clippy`

-   `expect()` and `unwrap()`:

    -   You should avoid using these methods as much as possible as they make the current thread panic. As a general rule, we want to handle error cases.
    -   There are some rare cases where `expect()` and `unwrap()` are suitable. Then, comment your code and share your reasoning with your co-contributors: why did you need and properly use these methods?
        > Hint: If you’re having trouble remembering how to phrase expect error messages remember to focus on the word “should” as in “env variable should be set by blah” or “the given binary should be available and executable by the current user”.
        -   Example - a specific environment variable is essential to your code's execution:
        ```rust
        let path = std::env::var("IMPORTANT_PATH").expect("env variable `IMPORTANT_PATH` should be set by `wrapper_script.sh`");
        ```

-   [Error handling](https://doc.rust-lang.org/beta/core/error/trait.Error.html): Rust enables a comprehensive Error handling flow. One can define structs/enums items that implement the trait Error. Then, any third party library or code higher in the execution flow can catch and respond properly to failed states. - In this project, we use [thiserror](https://crates.io/crates/thiserror/1.0.24) to describe and construct errors. A good example of this is [compile.rs](./src/compile.rs).

        ```
            #[derive(Error, Debug)]
            pub enum Error {
                #[error("binary '{CAIRO_COMPILE_BINARY}' not found{0}")]
                CairoCompileBinaryNotFound(#[from] WhichError),

                #[error("failed to execute a process: {0}")]
                RunProcess(io::Error),

                #[error("binary '{0}' failed to compile '{1}'")]
                Compilation(String, String),

                #[error("file '{0}' has no stem")]
                StemlessFile(String),

                #[error("cache directory does not exist on thiplatform")]
                CacheDirSupported,

                #[error("failed to create file '{0}': {1}")]
                FileCreation(String, io::Error),

                #[error("failed to create  directory '{0}': {1}")]
                DirCreation(String, io::Error),

                #[error("failed to write to file '{0}': {1}")]
                WriteToFile(String, io::Error),
            }
        ```

        -   One cool feature is to be able to raise _transparent_ errors:

            > Errors may use error(transparent) to forward the source and source methods straight through to an underlying error without adding an additional message. This would be appropriate for enums that need an "anything else" variant.

            ```rust
                #[derive(Error, Debug)]
                pub enum MyError {
                    #[error(transparent)]
                        Other(#[from] anyhow::Error),  // source and Display delegate to anyhow::Error
                }
            ```
            This is appropriate for enums that need an "anything else"/other variant. It's also useful when your enum variant can be constructed from multiple types of error. It is the equivalent of `Box<dyn Enum>`

For more, refer to [the Idiomatic Rust repository](https://github.com/mre/idiomatic-rust). It regroups all the good practices and elegant flows that Rust enables us to write. For instance, [error handling is described in this article](https://edgarluque.com/blog/wrapping-errors-in-rust/), written by Edgar Luque.

</br>

To build the project, run the following command:

```bash
cargo build
```

To launch all tests, run:

```bash
cargo test
```
