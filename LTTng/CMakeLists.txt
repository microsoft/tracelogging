cmake_minimum_required(VERSION 3.20)

project(tracelogging VERSION 0.2)

include(CTest)

add_compile_options(-Wall -Wextra)

list(APPEND CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake/modules)

find_package(LTTngUST REQUIRED)
find_package(lttng-ust-tracepoint REQUIRED)

# Include source
add_subdirectory(src)

# Use the standard BUILD_TESTING option to disable
add_subdirectory(test)
