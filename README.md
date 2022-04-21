# Cortex M Target Tester

`target_tester` is an ulta small library for running unit tests directly on a Cortex-M based target. The test code is
intended to be [googletest](https://github.com/google/googletest) compatible. As such, you **run the same tests on the
target**.

The test are run using the debug probe, thanks to the very awesome [probe-rs project](https://probe.rs/). When testing
on the target you only run a very small "test runtime". A test runner on the host then communicates with the target
using JTAG or SWD and tells the runtime which test to run. After each test, the target is reset and thus the test each
time starts with in a clean environment.

The *partial* source-compatibility to gtest allows you to write tests that run on the host as well as on the target.
This has various advantages:

* You can debug tests on the host
* You can run all sorts of tooling that is easy to run on the host but not so much on the target, such as code coverage
  analysis
* It gives you some additional confidence in your toolchain and that you are not exploiting some undefined behavior of
  the compiler.

## Getting Started

* Add the `target_test.{c,h}` to your project
* In your `main()` function, call `target_test_run_with_debugger()`, like so:

```c
#include "target_tester.h"

int main() {
    target_test_run_with_debugger();
    
    while (1) {}
}
```

* Write your tests as you normally would with gtest:

```c++

TEST(test_suite, test_name) {
    ASSERT_TRUE(false);
}
```

* Then create a config file that defines your target and how to access it:

```shell
$ cat target_test_config.json
{
  "interface": "SWD", # or JTAG
  "target": "STM32F105"
}
```

* Run the test with the test runner:

```shell
./target_tester --config target_test_config.json --output results.xml build/my-test-app 
```

The `--output` argument is optional. It allows exporting JUnit XML results for integration with CI.

## More Configuration Features

* Specify the communication speed of the debug probe: `"speed": {"KHz": 4000}`
* Add `probe-rs` target descriptions: `"target_description": "path/to/target_description.yaml"`. Note that the path is
  evaluated relative to the path of the config file. For more infor
* A vector table offset:`vector_table_offset: 1234`. Depending on your target or if you have a bootloader that you would
  like to skip you can provide the offset of the vector table in flash. The stack pointer (SP) and program counter (PC),
  will be initialized based on the values found in the vector table

## Building the test runner

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT) at your option.

## Acknowledgements

A big thank you to the [`probe-rs`](https://github.com/probe-rs/probe-rs) developers for building such an awesome
library. Implementing this would have been much more time-consuming without it.