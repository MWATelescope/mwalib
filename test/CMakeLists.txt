#CMakeLists.txt
cmake_minimum_required(VERSION 3.5)
project(mwalib_test)

add_definitions(-D_GNU_SOURCE=1)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_HOME_DIRECTORY}/bin)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

include_directories(${CMAKE_SOURCE_DIR}) # -I flags for compiler

set(PROGSRC src/main.c)            # define sources

IF(CMAKE_COMPILER_IS_GNUCXX)
    set(CMAKE_CXX_FLAGS_DEBUG "-g")
    set(CMAKE_CXX_FLAGS_RELEASE "-O3")
    SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Werror -Wextra")
    SET(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Werror -Wextra")    
ENDIF(CMAKE_COMPILER_IS_GNUCXX)

add_executable(mwalib_test ${PROGSRC})       # define executable target prog, specify sources
target_link_libraries(mwalib_test LINK_PUBLIC mwalib pthread cfitsio)   # -l flags for linking target
