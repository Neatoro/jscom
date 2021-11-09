cmake_minimum_required(VERSION 3.10)

project(compiler)

add_custom_target(build ALL)
add_custom_command(
    TARGET build
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

set(JSCOM_ARTIFACT "app")

if(CMAKE_SYSTEM_NAME STREQUAL "Windows")
    set(JSCOM_ARTIFACT "app.exe")
endif()

install(
    PROGRAMS ${CMAKE_CURRENT_SOURCE_DIR}/target/release/${JSCOM_ARTIFACT}
    DESTINATION ${CMAKE_INSTALL_PREFIX}
)
