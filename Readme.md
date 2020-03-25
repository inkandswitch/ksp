# The Knowledge Server

We want to discover & surface connections between ideas, documents and concepts. This ability is something other tools have specialized in but each within their own silos. By creating an index of that knowledge for a user, we can give them the superpower to discover connections among their own work where ever they go.

### Implementation

This repository is reference implementation in Rust. It exposes [GraphQL][] API
so that other tools can interface to submit / discover connections between
resources identified by URLs (Local resources are represented via file:/// URLs).

### Usage

At the moment no binaries are distributed, however you can build / run using
[cargo][].

```sh
cargo run
```

Once running you can explore protocol schema, execute queries / mutations using
GraphQL IDE from http://localhost:8080/graphiql

[cargo]: https://doc.rust-lang.org/cargo/ "Rust package manager"
[graphql]: https://graphql.org/ "A query language for your API"
