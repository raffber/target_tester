#ifndef TARGET_TEST_H
#define TARGET_TEST_H

#include "stdint.h"

#define TARGET_TEST_RUNTIME_GTEST  1
#define TARGET_TEST_RUNTIME_NATIVE 2

#if TARGET_TEST_RUNTIME == TARGET_TEST_RUNTIME_GTEST
#include "gtest/gtest.h"
#elif TARGET_TEST_RUNTIME == TARGET_TEST_RUNTIME_NATIVE

#ifdef __cplusplus
extern "C" {
#define TARGET_TEST_ABI extern "C"
#else
#define TARGET_TEST_ABI
#endif

#define TARGET_TEST_FUN_NAME(suite_name, test_name) target_test_test_##suite_name##__target_test__##test_name

#define TARGET_TEST_CALL(suite_name, test_name)                                                                        \
    void TARGET_TEST_FUN_NAME(suite_name, test_name)();                                                                \
    TARGET_TEST_FUN_NAME(suite_name, test_name)();

void target_test_startup();
void target_test_run_with_debugger();

void target_test_fail(const char *file_path, uint32_t lineno);
void target_test_fail_with_reason(const char *file_path, uint32_t lineno, int32_t reason);

#define TEST(suite_name, test_name)                                                                                    \
    TARGET_TEST_ABI __attribute__((section(".target_test"))) void TARGET_TEST_FUN_NAME(suite_name, test_name)(void)

#define ASSERT_EQ(lhs, rhs)                                                                                            \
    do {                                                                                                               \
        if ((lhs) == (rhs)) {                                                                                          \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 1);                                                           \
    } while (0);

#define ASSERT_TRUE(value)                                                                                             \
    do {                                                                                                               \
        if ((value)) {                                                                                                 \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 2);                                                           \
    } while (0);

#define ASSERT_FALSE(value)                                                                                            \
    do {                                                                                                               \
        if (!(value)) {                                                                                                \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 3);                                                           \
    } while (0);

#define ASSERT_LE(lhs, rhs)                                                                                            \
    do {                                                                                                               \
        if ((lhs) < (rhs)) {                                                                                           \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 4);                                                           \
    } while (0);

#define ASSERT_LEQ(lhs, rhs)                                                                                           \
    do {                                                                                                               \
        if ((lhs) <= (rhs)) {                                                                                          \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 5);                                                           \
    } while (0);

#define ASSERT_GE(lhs, rhs)                                                                                            \
    do {                                                                                                               \
        if ((lhs) > (rhs)) {                                                                                           \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 6);                                                           \
    } while (0);

#define ASSERT_GEQ(lhs, rhs)                                                                                           \
    do {                                                                                                               \
        if ((lhs) > (rhs)) {                                                                                           \
            break;                                                                                                     \
        }                                                                                                              \
        target_test_fail_with_reason(__FILE__, __LINE__, 7);                                                           \
    } while (0);

#ifdef __cplusplus
}// extern "C"
#endif

#else
#error("Invalid or no TARGET_TEST_RUNTIME selected.")
#endif

#endif// TARGET_TEST_H