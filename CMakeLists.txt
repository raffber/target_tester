cmake_minimum_required(VERSION 3.22)
project(target_tester C CXX)

set(CMAKE_C_STANDARD 11)

include_directories(.)
include_directories(src)

add_executable(target_tester
        src/linux/main.cpp
        src/linux/test.c
        src/target_test.h
        src/target_test.c
        )

target_compile_definitions(target_tester PRIVATE TARGET_TEST_RUNTIME=2)