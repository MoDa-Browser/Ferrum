cmake_minimum_required(VERSION 3.20)
project(moda-examples)

add_executable(minimal-browser
    minimal_browser.cpp
)

target_link_libraries(minimal-browser
    PRIVATE
        moda-render
)
