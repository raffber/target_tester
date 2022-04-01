#include "target_test.h"

#include "stdbool.h"
#include "stddef.h"

#ifndef TARGET_TEST_ABORT
#define TARGET_TEST_ABORT abort();
#endif

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
__attribute__((section(".target_test"))) static __attribute__((noreturn)) void abort() {
    while (1) {
    }
}

#endif

typedef void (*target_test_voidfun_t)(void);

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

#define MEMORY_SYNC                                                                                                    \
    do {                                                                                                               \
        __sync_synchronize();                                                                                          \
    } while (0);

__attribute__((section(".target_test"))) static uint32_t target_test_crc32(const volatile void *data, uint32_t size) {
    uint32_t crc = ~0;
    volatile uint8_t *byte_data = (volatile uint8_t *) data;

    for (uint32_t k_byte = 0; k_byte < size; ++k_byte) {
        crc ^= byte_data[k_byte];
        for (uint32_t k_bit = 0; k_bit < 8; ++k_bit) {
            uint32_t temp = ~((crc & 1) - 1);
            crc = (crc >> 1) ^ (0xEDB88320 & temp);// reflection of 0x4C11DB7
        }
    }

    return ~crc;
}

__attribute__((section(".target_test"))) static void target_test_state_data_update(target_test_state_t new_state) {
    target_test_data.state = new_state;
    target_test_data.crc = target_test_crc32(&target_test_data, sizeof(target_test_data) - sizeof(uint32_t));
    MEMORY_SYNC
}

__attribute__((constructor)) __attribute__((section(".target_test"))) void target_test_startup() {
    target_test_state_data_update(TARGET_TEST_IDLE);
}

__attribute__((section(".target_test"))) void target_test_run_with_debugger() {
    target_test_state_data_update(TARGET_TEST_READY);

    while (target_test_fun_to_run == NULL) {
    }
    MEMORY_SYNC

    target_test_data.executed_function = target_test_fun_to_run;
    target_test_state_data_update(TARGET_TEST_STARTED);
    target_test_fun_to_run();
    if (target_test_data.state != TARGET_TEST_FAILED) {
        target_test_state_data_update(TARGET_TEST_PASSED);
    }
}

__attribute__((section(".target_test"))) void
target_test_fail_with_reason(const char *file_path, uint32_t lineno, int32_t reason) {
    MEMORY_SYNC

    target_test_data.test_file_path = file_path;
    target_test_data.lineno = lineno;
    target_test_data.fail_reason = (target_test_assertion_type_t) reason;
    target_test_state_data_update(TARGET_TEST_FAILED);

    TARGET_TEST_ABORT
}

__attribute__((section(".target_test"))) void target_test_fail(const char *file_path, uint32_t lineno) {
    target_test_fail_with_reason(file_path, lineno, TARGET_TEST_ASSERT_NONE);
}
