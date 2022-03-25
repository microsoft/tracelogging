cmake_minimum_required(VERSION 3.5)

add_executable(
    traceloggingTest
    CatchMain.cpp
    TestTraceLogging.cpp
    TestTraceLogging.c)

target_link_libraries(
    traceloggingTest
    PRIVATE
    tracelogging
    # lttng-ust-common # Needed for LTTNG 2.13+
    Catch2::Catch2)

include(CTest)
include(Catch)
catch_discover_tests(traceloggingTest)
