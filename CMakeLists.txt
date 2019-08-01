cmake_minimum_required(VERSION 3.5)

project(tracelogging)

include(GNUInstallDirs)

add_compile_options(-Wall -Wextra -Werror)
set(TRACELOGGING_BUILD_TESTS ON CACHE BOOL "Build tests.")

# Include source
add_subdirectory(src)

# Only include testing stuff if we are the top level
if(TRACELOGGING_BUILD_TESTS)
    if (${CMAKE_PROJECT_NAME} STREQUAL ${PROJECT_NAME})
        enable_testing()

        if (NOT TARGET Catch2::Catch2)
            list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/external/Catch2/contrib")
            add_subdirectory(external/Catch2)
        endif ()

        add_subdirectory(test)
    endif ()
endif ()
