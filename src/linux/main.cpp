#include "target_test.h"

extern "C" void target_test_test_foo__target_test__bar(void);

int main() {
//    target_test_test_foo__target_test__bar();
    target_test_run_all();
    return 0;
}