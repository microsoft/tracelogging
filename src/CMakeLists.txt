cmake_minimum_required(VERSION 3.5)

set(TLG_INCLUDEDIR ${PROJECT_SOURCE_DIR}/include/)

# Add source to this project's executable.
add_library(lttngh
    LttngActivityHelpers.c
    LttngHelpers.c
    LttngNetHelpers.c)

target_compile_features(lttngh PRIVATE c_std_99)

target_include_directories(lttngh PUBLIC $<INSTALL_INTERFACE:include> $<BUILD_INTERFACE:${TLG_INCLUDEDIR}>)
target_link_libraries(lttngh PUBLIC LTTng::UST PRIVATE lttng-ust-tracepoint::lttng-ust-tracepoint)

set_property(TARGET lttngh PROPERTY POSITION_INDEPENDENT_CODE ON)
set_property(TARGET lttngh PROPERTY SOVERSION 0)

add_library(tracelogging INTERFACE)
target_include_directories(tracelogging INTERFACE $<INSTALL_INTERFACE:include> $<BUILD_INTERFACE:${TLG_INCLUDEDIR}>)
target_link_libraries(tracelogging INTERFACE lttngh)

set(TARGETLIST tracelogging lttngh)

install(DIRECTORY ${TLG_INCLUDEDIR}
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
install(TARGETS ${TARGETLIST}
    EXPORT tracelogging-export
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    INCLUDES DESTINATION ${CMAKE_INSTALL_LIBDIR})

add_library(tracelogging::tracelogging ALIAS tracelogging)

set(TRACELOGGING_CMAKE_DIR ${CMAKE_INSTALL_LIBDIR}/cmake/tracelogging)

install(
    EXPORT tracelogging-export
    FILE traceloggingTargets.cmake
    NAMESPACE tracelogging::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/tracelogging)

install(FILES ${PROJECT_SOURCE_DIR}/cmake/modules/traceloggingConfig.cmake
    DESTINATION ${TRACELOGGING_CMAKE_DIR})
