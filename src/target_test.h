#ifndef TARGET_TEST_H
#define TARGET_TEST_H

typedef void (*test_voidfun_t)(void);

#define TEST_FUN_NAME(suite_name, test_name)        __test_ ## suite_name ## _ ## test_name

#define TEST(suite_name, test_name) \
    void TEST_FUN_NAME(suite_name, test_name)(); \
    volatile test_voidfun_t __ptr__test_ ## suite_name ## _ ## test_name = &TEST_FUN_NAME(suite_name, test_name); \
                                            \
    void TEST_FUN_NAME(suite_name, test_name)()

#endif // TARGET_TEST_H