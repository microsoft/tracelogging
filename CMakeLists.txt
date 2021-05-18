cmake_minimum_required(VERSION 3.6)

project(tracelogging VERSION 0.2)

include(GNUInstallDirs)

add_compile_options(-Wall -Wextra)

list(APPEND CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake/modules)

find_package(LTTngUST REQUIRED)
find_package(lttng-ust-tracepoint REQUIRED)

# Include source
add_subdirectory(src)

# Only include testing stuff if we are the top level and someone hasn't disabled them explicitly
option(TRACELOGGING_BUILD_TESTS "Set to OFF to disable building of tests" ON)

if (TRACELOGGING_BUILD_TESTS)
    if (${CMAKE_PROJECT_NAME} STREQUAL ${PROJECT_NAME})
        enable_testing()

        if (EXISTS ${PROJECT_SOURCE_DIR}/external/Catch2/CMakeLists.txt)
            list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/external/Catch2/contrib")
            add_subdirectory(external/Catch2)
        else ()
            find_package(Catch2 REQUIRED)
        endif ()

        add_subdirectory(test)
    endif ()
endif ()
