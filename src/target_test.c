#include "target_test.h"

#include "stdbool.h"
#include "stddef.h"

// TODO: consider allowing to define a weak abort() symbol

#ifdef _WIN32
#include <windows.h>

static __attribute__((noreturn)) void abort() {
    DebugBreak();
    exit(1);
}
#elif __unix__
    // include abort()
    #include "stdlib.h"
#else

// embedded targets
static __attribute__((noreturn)) void abort() {
    while (1) {}
}

#endif


typedef enum {
    TARGET_TEST_ASSERT_NONE = 0,
    TARGET_TEST_ASSERT_EQ = 1,
    TARGET_TEST_ASSERT_TRUE = 2,
    TARGET_TEST_ASSERT_FALSE = 3,
} target_test_assertion_type_t;

typedef enum {
    TARGET_TEST_IDLE = 0x8C3F82FA,
    TARGET_TEST_READY = 0xD79A2E5F,
    TARGET_TEST_STARTED = 0xCD833CB7,
    TARGET_TEST_PASSED = 0xBAF2C481,
    TARGET_TEST_FAILED = 0xCA83D14E,
} target_test_state_t;


typedef struct {
    target_test_state_t state;
    target_test_voidfun_t executed_function;
    target_test_assertion_type_t fail_reason;
    const char *test_file_path;
    uint32_t lineno;
    uint32_t crc;
} target_test_data_t;

volatile target_test_data_t target_test_data;
volatile target_test_voidfun_t target_test_fun_to_run;

target_test_registered_test_t *target_test_registry;

#define MEMORY_SYNC         do { \
    for (volatile int k = 0; k < 10; ++k) {}  \
} while(0);

uint32_t target_test_crc32(const volatile void *data, uint32_t size) {
    uint32_t crc = ~0;
    volatile uint8_t * byte_data = (volatile uint8_t *)data;

    for (uint32_t k_byte = 0; k_byte < size; ++k_byte) {
        crc ^= byte_data[k_byte];
        for (uint32_t k_bit = 0; k_bit < 8; ++k_bit) {
            uint32_t temp = ~((crc & 1) - 1);
            crc = (crc >> 1) ^ (0x4C11DB7 & temp);
        }
    }

    return ~crc;
}

static void target_test_state_data_update(target_test_state_t new_state) {
    target_test_data.state = new_state;
    target_test_data.crc = target_test_crc32(&target_test_data, sizeof(target_test_data) - sizeof(uint32_t));
    MEMORY_SYNC
}

void target_test_run_with_debugger() {
    target_test_state_data_update(TARGET_TEST_READY);

    while (target_test_fun_to_run == NULL) {}
    MEMORY_SYNC

    target_test_state_data_update(TARGET_TEST_STARTED);
    target_test_fun_to_run();
    target_test_state_data_update(TARGET_TEST_PASSED);
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
