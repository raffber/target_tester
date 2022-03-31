#include <stdio.h>
#include "stdbool.h"
#include "target_test.h"

TEST(foo, bar) {
    printf("foo.bar\n");
}

TEST(bar, baz) {
    printf("bar.baz\n");
    ASSERT_TRUE(false);
}

