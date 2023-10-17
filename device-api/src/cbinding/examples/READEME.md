### build examples
1. Build device-api with `cbinding` and `blocking` features in the root directory of git project.
   ```bash
   cargo build --features cbinding,blocking
   ```

2. Install shared library under /user/local/lib
   ```bash
   sudo cp target/debug/libfuriosa_device.so /user/local/lib
   ```

3. move to the example directory and build object file then link to shared library.
   ```bash
   cd {GIT_PROJECT_ROOT}}/device-api/src/cbinding/examples
   gcc -c {EXAMPLE_SOURCE_FILE_NAME} -o {TARGET_OBJECT_FILE}
   gcc {TARGET_OBJECT_FILE}.o -L/user/local/lib/ -lfuriosa_device -o {TARGET_BINARY_FILE}
   ```
4. Execute binary
   ```bash
   ./{TARGET_BINARY_FILE}
   ```