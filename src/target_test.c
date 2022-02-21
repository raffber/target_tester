#include "target_test.h"

#include "stdbool.h"
#include "stddef.h"

// TODO: consider allowing to define a weak abort() symbol

#ifdef _WIN32
#include <windows.h>

static void abort() {
    DebugBreak();
    exit(1);
}
#elif __unix__
    // include abort()
    #include "stdlib.h"
#else

// embedded targets
static void abort() {
    while (1) {}
}

#endif


typedef enum {
    TARGET_TEST_ASSERT_NONE = 0,
    TARGET_TEST_ASSERT_EQ = 1,
    TARGET_TEST_ASSERT_TRUE = 2,
    TARGET_TEST_ASSERT_FALSE = 3,
} target_test_assertion_type_t;


volatile target_test_voidfun_t target_test_fun_to_run;
volatile bool target_test_started;
volatile bool target_test_done;
volatile bool target_test_passed;
volatile const char *target_test_file_path;
volatile uint32_t target_test_lineno;

volatile int32_t target_test_fail_reason;
volatile uint64_t target_test_reason_data[2];

target_test_registered_test_t *target_test_registry;

#define MEMORY_SYNC         do { \
    for (volatile int k = 0; k < 10; ++k) {}  \
} while(0);

void target_test_run_with_debugger() {
    while (target_test_fun_to_run == NULL) {}
    MEMORY_SYNC

    target_test_started = true;

    MEMORY_SYNC
    target_test_fun_to_run();
    MEMORY_SYNC

    target_test_passed = true;
    target_test_done = true;

    MEMORY_SYNC
}

void target_test_run_all() {
    target_test_registered_test_t *reg = target_test_registry;
    if (reg == NULL) {
        return;
    }

    target_test_started = true;

    do {
        reg->fun();
        reg = reg->next;
    } while (reg != NULL);

    target_test_done = true;
}

void target_test_fail_with_reason(const char *file_path, uint32_t lineno, int32_t reason, const uint64_t reason_data[2]) {
    MEMORY_SYNC

    target_test_file_path = file_path;
    target_test_lineno = lineno;
    target_test_passed = false;
    target_test_done = true;
    target_test_fail_reason = reason;
    if (reason_data != NULL) {
        target_test_reason_data[0] = reason_data[0];
        target_test_reason_data[1] = reason_data[1];
    }

    MEMORY_SYNC

    abort();
}

void target_test_fail(const char *file_path, uint32_t lineno) {
    target_test_fail_with_reason(file_path, lineno, TARGET_TEST_ASSERT_NONE, NULL);
}

void target_test_register(target_test_registered_test_t *test, target_test_voidfun_t fun) {
    test->fun = fun;
    test->next = NULL;

    if (target_test_registry == NULL) {
        target_test_registry = test;
        return;
    }

    target_test_registered_test_t *reg = target_test_registry;
    while (reg->next != NULL) {
        reg = reg->next;
    }
    reg->next = test;
}
