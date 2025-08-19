#include <gtest/gtest.h>

extern "C" {
    #include "math.h"
}

TEST(MathTest, Add) {
    EXPECT_EQ(add(2, 3), 5);
    EXPECT_EQ(mul(2, 3), 6);
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
