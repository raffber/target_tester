cmake_minimum_required(VERSION 3.22)
include(FetchContent)
FetchContent_Declare(
        googletest
        URL https://github.com/google/googletest/archive/609281088cfefc76f9d0ce82e1ff6c30cc3591e5.zip
)
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

project(target_tester C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 11)

enable_testing()

include_directories(.)
include_directories(src)

add_executable(target_tester
        src/linux/test.cpp
        src/target_test.h
        )

target_compile_definitions(target_tester PRIVATE TARGET_TEST_RUNTIME=1)
target_link_libraries(target_tester gtest_main)


