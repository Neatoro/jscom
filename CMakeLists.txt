cmake_minimum_required(VERSION 3.10)

project(jscom)

add_subdirectory(compiler)
add_subdirectory(lib)

set(LLVM_ENABLE_PROJECTS "clang;libc" CACHE STRING "LLVM Enable Projects")
set(CMAKE_INSTALL_PREFIX ${CMAKE_INSTALL_PREFIX} CACHE STRING "Prefix")


add_subdirectory(llvm-project/llvm)
