# see-seeker

Quickly generate templates based on github repo or gitlab repo.

[![NPM version][npm-image]][npm-url]

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).

## Installing see-seeker

```sh
$ npm install see-seeker -g
```

## Exploring see-seeker

After install see-seeker, you can explore it at the Node REPL:

```sh
$ npm install see-seeker -g
$ see https://github.com/Dlouxgit/see ./see
```

## Available Scripts

In the project directory, you can run:

### `see [select [dest]]`

Pull items from the local list to the dest directory, dest defaults to the current directory.

### `see set-token <access_token>`

Set gitlab's private token for internal projects.

### `see <repo-url> [dest]`

Only record the repo url and do not pull it.

### `see add <repo-url>`

Pull the project of repo-url to the directory corresponding to dest, dest is the current directory by default.

## Project Layout

The directory structure of this project is:

```
see/
├── Cargo.toml
├── README.md
├── index.node
├── package.json
├── src/
|   └── lib.rs
├── bin/
|   └── main.js
└── target/
```

### Cargo.toml

The Cargo [manifest file](https://doc.rust-lang.org/cargo/reference/manifest.html), which informs the `cargo` command.

### README.md

This file.

### index.node

The Node addon—i.e., a binary Node module—generated by building the project. This is the main module for this package, as dictated by the `"main"` key in `package.json`.

Under the hood, a [Node addon](https://nodejs.org/api/addons.html) is a [dynamically-linked shared object](https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries). The `"build"` script produces this file by copying it from within the `target/` directory, which is where the Rust build produces the shared object.

### package.json

The npm [manifest file](https://docs.npmjs.com/cli/v7/configuring-npm/package-json), which informs the `npm` command.

### src/

The directory tree containing the Rust source code for the project.

### src/lib.rs

The Rust library's main module.

### bin/main.js

The cli main file.

### target/

Binary artifacts generated by the Rust build.

## Learn More

To learn more about Neon, see the [Neon documentation](https://neon-bindings.com).

To learn more about Rust, see the [Rust documentation](https://www.rust-lang.org).

To learn more about Node, see the [Node documentation](https://nodejs.org).


## LICENSE
ISC

[npm-image]: https://img.shields.io/npm/v/see-seeker.svg?style=flat-square
[npm-url]: https://npmjs.org/package/see-seeker