#include "target_test.h"

extern "C" {
// for testing disable abort()
#define TARGET_TEST_ABORT
#include "target_test.c"
}

#include <vector>

TEST(target_test, crc32) {
    std::vector<uint8_t> data = {0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39};
    uint32_t crc = target_test_crc32(data.data(), data.size());
    ASSERT_EQ(crc, 0xCBF43926);
}

namespace {

bool successful_test_fun_executed;

void successful_test_function() {
    successful_test_fun_executed = true;
}

bool failing_test_fun_executed;

void failing_test_function() {
    failing_test_fun_executed = true;
    target_test_fail_with_reason(__FILE__, __LINE__, TARGET_TEST_ASSERT_EQ);
}

void check_test_data_struct_integrity() {
    uint32_t ref_crc = target_test_crc32(&target_test_data, sizeof(target_test_data) - sizeof(uint32_t));
    ASSERT_EQ(ref_crc, target_test_data.crc);
}

}// namespace

TEST(target_test, check_startup) {
    target_test_startup();

    ASSERT_EQ(target_test_data.test_file_path, nullptr);
    ASSERT_EQ(target_test_data.executed_function, nullptr);
    ASSERT_EQ(target_test_data.fail_reason, TARGET_TEST_ASSERT_NONE);
    ASSERT_EQ(target_test_data.lineno, 0);
    ASSERT_EQ(target_test_data.state, TARGET_TEST_IDLE);
    check_test_data_struct_integrity();
}

TEST(target_test, pass_test) {
    successful_test_fun_executed = false;
    target_test_startup();

    target_test_fun_to_run = successful_test_function;
    target_test_run_with_debugger();

    ASSERT_TRUE(successful_test_fun_executed);

    ASSERT_EQ(target_test_data.test_file_path, nullptr);
    ASSERT_EQ(target_test_data.executed_function, successful_test_function);
    ASSERT_EQ(target_test_data.fail_reason, TARGET_TEST_ASSERT_NONE);
    ASSERT_EQ(target_test_data.lineno, 0);
    ASSERT_EQ(target_test_data.state, TARGET_TEST_PASSED);
    check_test_data_struct_integrity();
}

TEST(target_test, fail_test) {
    failing_test_fun_executed = false;

    target_test_startup();
    target_test_fun_to_run = failing_test_function;

    target_test_run_with_debugger();

    ASSERT_TRUE(failing_test_fun_executed);

    ASSERT_EQ(target_test_data.test_file_path, __FILE__);
    ASSERT_EQ(target_test_data.executed_function, failing_test_function);
    ASSERT_EQ(target_test_data.fail_reason, TARGET_TEST_ASSERT_EQ);
    ASSERT_EQ(target_test_data.lineno, 29);// this is a little shitty, but well...
    ASSERT_EQ(target_test_data.state, TARGET_TEST_FAILED);
    check_test_data_struct_integrity();
}
