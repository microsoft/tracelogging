get_filename_component(TRACELOGGING_CMAKE_DIR "${CMAKE_CURRENT_LIST_FILE}" PATH)
include(CMakeFindDependencyMacro)

find_dependency(LTTngUST REQUIRED)

if (NOT TARGET tracelogging::tracelogging)
    include("${TRACELOGGING_CMAKE_DIR}/traceloggingTargets.cmake")
endif ()
