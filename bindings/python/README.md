# Furiosa Device Python API

Python binding of Furiosa Device API.

## Documentations
You can render docs by `pydoc` or `pdoc`. However, it is just a binding of `device-api`, so please see the [Rust documentation](../../README.md) for a mode detailed explanation.

## Develop environment with maturin

To use maturin, you need to use a python virtual environment. A simple example can be found [here](https://github.com/PyO3/pyo3/#Usage).

```
pip install maturin
make develop
```

## Test

```
make test
```

## Examples

 Some examples for using Python API is available [here](examples).