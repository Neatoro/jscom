cmake_minimum_required(VERSION 3.10)

project(libs)

include_directories(include)

add_library(log STATIC src/log.c)
install(
    TARGETS log
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib
)
