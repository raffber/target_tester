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
  "interface": "SWD",
  "speed": {
    "KHz": 4000
  },
  "target": "STM32F105"
}
```

* Run the test with the test runner:

```shell
./target_tester --config target_test_config.json build/my-test-app
```

## More Features

## Building the test runner



