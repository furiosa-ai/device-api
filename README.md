# Furiosa Device API

A set of APIs to list and retrieve information of FuriosaAI's NPU devices. To learn more about FuriosaAI's NPU, please visit <https://furiosa.ai>.

# Before you start

This crate requires FuriosaAI's NPU device and its kernel driver. Currently, FuriosaAI offers NPU devices for only users who register Early Access Program (EAP). Please contact <contact@furiosa.ai> to learn how to start the EAP. You can also refer to [Driver, Firmware, and Runtime Installation](https://furiosa-ai.github.io/docs/latest/en/software/installation.html) to learn the kernel driver installation.

# Documentation for Rust

To render docs in local, please use a command as below (*requires nightly):
```bash
cd device-api
cargo rustdoc --lib --all-features -- --cfg docsrs
```

# For Python

The Python API for FuriosaAI's NPU device is available [here](device-api-python/).

# License

```
Copyright 2022 FuriosaAI, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
