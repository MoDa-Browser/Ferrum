cmake_minimum_required(VERSION 3.20)
project(MoDaCore VERSION 0.1.0 LANGUAGES CXX Rust)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Wpedantic")

option(MODA_BUILD_TESTS "Build tests" ON)
option(MODA_BUILD_EXAMPLES "Build examples" ON)
option(MODA_ENABLE_LTO "Enable link-time optimization" OFF)

find_package(Rust_CARGO REQUIRED)

add_subdirectory(src/render)

if(MODA_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

if(MODA_BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()
