# Development requirements

To develop this project you need to have the following tools installed:
- [Rust](https://www.rust-lang.org/tools/install) - required for building the project;
- [Just](https://just.systems/man/en) - optional, but recommended;
- [Docker](https://docs.docker.com/get-docker) - optional, but recommended (different container runtime can be used, but requires manual configuration).

# Building the project

To build the project run the following command:
```sh
just build
```

For building in release mode run:
```sh
just release=true build
```

# Building the release Docker image

To build the release Docker image run the following command:
```sh
just image_build
```

For building the image with a specific tag run:
```sh
just image_name=<name> image_tag=<tag> image_build
```

# Importing/exporting the release Docker image

To export the release Docker image run the following command:
```sh
just image_save
```

And archive will be created in the project directory.

To import the release Docker image run the following command:
```sh
just image_load
```

To import/export archive with a specific name run:
```sh
just image_archive=<name> [image_save|image_load]
```

# Pull requests requirements

Before sending a pull request make sure you do the following:
- run `just fmt` to format the code;
- run `just lint` to lint the code;
- run `just test_unit` to run the unit tests;
- run `just doc` to generate the documentation;
- run `just image_build` to build the Docker image.

Additionally formatting requirements:
- use 4 spaces for indentation;
- use 80 characters per line;
- all macro attributes (and their arguments) are sorted alphabetically;
- `mod`s are:
    - sorted alphabetically;
    - defined before `use`s (expect inlined `mod`s).
- `use`s are:
    - sorted alphabetically;
    - separated by a blank line between different groups (`std`, external-crates, local-crates).
- `pub use`s are:
    - sorted alphabetically;
    - separated by a blank line between different groups (`std`, external-crates, local-crates);
    - defined after the `use`s.

After all the checks pass, you can submit a pull request.

If you have any questions/suggestions, feel free to open an issue. 
