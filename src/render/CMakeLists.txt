cmake_minimum_required(VERSION 3.20)
project(moda-render VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Wpedantic")

add_library(moda-render STATIC
    src/layout_engine.cpp
    src/paint_engine.cpp
    src/dom_manager.cpp
)

target_include_directories(moda-render
    PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_link_libraries(moda-render
    PRIVATE
        stdc++
)
