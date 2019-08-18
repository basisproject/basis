# Basis

This is a blockchain implementation of the system described in the [basis paper](https://gitlab.com/basis-/paper). The system's processes and decisions will not be described here as the paper will do a better job. Here you will find technical information on using/running the project.

## Building

Requires Rust v1.34+. To build the project, run:

```
make
```

To configure, run:

```
make reconfig
```

Now run it:

```
make run
```

Congrats, you are running a Basis node.

## Testing

Basis comes with a series of built-in unit tests that can be run via:

```
make test
```

It also comes with a set of [integration tests as an included project](./integration-tests).


## License

Basis uses AGPLv3.0, which means not only is it a copyleft license, but anyone running the server must also provide/publish the source to users of their server on request.

Basis, at its core, is about complete transparency and openness. The AGPL license reflects and to some extent enforces this.



