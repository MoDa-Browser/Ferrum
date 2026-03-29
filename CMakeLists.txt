cmake_minimum_required(VERSION 3.20)
project(MoDaCore VERSION 0.1.0 LANGUAGES Rust)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(MODA_BUILD_TESTS "Build tests" ON)
option(MODA_BUILD_EXAMPLES "Build examples" ON)
option(MODA_ENABLE_LTO "Enable link-time optimization" OFF)

find_package(Rust_CARGO REQUIRED)

if(MODA_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

if(MODA_BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()
