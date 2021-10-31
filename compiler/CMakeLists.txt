cmake_minimum_required(VERSION 3.10)

project(compiler)

add_custom_target(build ALL)
add_custom_command(
    TARGET build
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

install(
    PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/target/release/app.exe
    DESTINATION ${CMAKE_INSTALL_PREFIX}
)
