cmake_minimum_required(VERSION 3.5)

add_executable(
    traceloggingTest
    CatchMain.cpp
    TestTraceLogging.cpp
    TestTraceLogging.c)

# FIXME: For LTTNG 2.13+, also needs lttng-ust-common
target_link_libraries(
    traceloggingTest
    PRIVATE
    tracelogging
    Catch2::Catch2)

include(CTest)
include(Catch)
catch_discover_tests(traceloggingTest)
