#ifndef TARGET_TEST_H
#define TARGET_TEST_H

#include "stdint.h"


#ifdef __cplusplus
extern "C" {
#endif

#define TARGET_TEST_REASON_USER     0x1000

typedef void (*target_test_voidfun_t)(void);

struct target_test_registered_test_s;
typedef struct target_test_registered_test_s target_test_registered_test_t;

struct target_test_registered_test_s {
    target_test_voidfun_t fun;
    target_test_registered_test_t *next;
};

#define TEST_FUN_NAME(suite_name, test_name)        target_test_test_ ## suite_name ## __target_test__ ## test_name
#define TEST_CONSTRUCTOR_NAME(suite_name, test_name)        target_test_constructor_ ## suite_name ## __target_test__ ## test_name


void target_test_run_with_debugger();

void target_test_run_all();

void target_test_register(target_test_registered_test_t *test, target_test_voidfun_t fun);

__attribute__((noreturn)) void target_test_fail(const char *file_path, uint32_t lineno);
__attribute__((noreturn)) void
target_test_fail_with_reason(const char *file_path, uint32_t lineno, int32_t reason, const uint64_t reason_data[2]);


#define TEST(suite_name, test_name) \
    void TEST_FUN_NAME(suite_name, test_name)(); \
    void __attribute__ ((constructor)) TEST_CONSTRUCTOR_NAME(suite_name, test_name)(void)  { \
        static target_test_registered_test_t reg; \
        target_test_voidfun_t fun = & TEST_FUN_NAME(suite_name, test_name); \
        target_test_register(&reg, fun); \
    }\
    void TEST_FUN_NAME(suite_name, test_name)(void)


#define ASSERT_EQ(lhs, rhs) \
    do {                    \
        if ((lhs) == (rhs)) { \
            break;                     \
        }                   \
        target_test_fail_with_reason(__FILE__, __LINE__, 1, NULL);                    \
    } while(0);

#define ASSERT_TRUE(value) \
    do {                    \
        if ((value)) { \
            break;                     \
        }                   \
        target_test_fail_with_reason(__FILE__, __LINE__, 2, NULL);                    \
    } while(0);

#define ASSERT_FALSE(value) \
    do {                    \
        if (!(value)) { \
            break;                     \
        }                   \
        target_test_fail_with_reason(__FILE__, __LINE__, 3, NULL);                    \
    } while(0);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // TARGET_TEST_H