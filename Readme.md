# The Knowledge Server

We want to discover & surface connections between ideas, documents and concepts. This ability is something other tools have specialized in but each within their own silos. By creating an index of that knowledge for a user, we can give them the superpower to discover connections among their own work where ever they go.

### Implementation

This repository is reference implementation in Rust. It exposes [GraphQL][] API
so that other tools can interface to submit / discover connections between
resources identified by URLs (Local resources are represented via file:/// URLs).

### Usage

At the moment no binaries are distributed, however you can build / run using
[cargo][]. To create a (debug) build run:

```sh
cargo +nightly build
```

By default, the build will produce a binary at `./target/debug/knowledge-server`.

#### Server

You can start a knowledge-server by running:

```sh
./target/debug/knowledge-server serve
```

Once it's running you can explore protocol schema, execute queries / mutations
using GraphQL IDE at http://localhost:8080/graphiql. 

(This won't be very interesting at first, as you won't have submitted any data to it.)

#### Daemon

You can spawn a knowledge-server as a daemon by runing:

```sh
./target/debug/knowledge-server daemon
```

#### Scan / Ingest content

You can ingest local markdown files into your knowledge base
by running:

```sh
./target/debug/knowledge-server scan $YOUR_PATH_HERE
```

### Hacking Notes

We ran into [issue][rust-lang/rls-vscode#755] with [rls-vscode][] extension. If
you use vscode you may want to consider [rust analyzer][] instead.

[rust-lang/rls-vscode#755]: https://github.com/rust-lang/rls-vscode/issues/755
[cargo]: https://doc.rust-lang.org/cargo/ 'Rust package manager'
[graphql]: https://graphql.org/ 'A query language for your API'
[rls-vscode]: https://github.com/rust-lang/rls-vscode 'Rust support for Visual Studio Code'
[rust analyzer]: https://rust-analyzer.github.io/
