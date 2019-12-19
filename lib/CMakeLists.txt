#CMakeLists.txt
cmake_minimum_required(VERSION 3.5)
project(mwalib)

add_definitions(-D_GNU_SOURCE=1)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_HOME_DIRECTORY}/lib)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

set(PROGSRC src/args.c src/fitsreader.c)            # define sources

IF(CMAKE_COMPILER_IS_GNUCXX)
    set(CMAKE_CXX_FLAGS_DEBUG "-g")
    set(CMAKE_CXX_FLAGS_RELEASE "-O3")
    SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Werror -Wextra")
    SET(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Werror -Wextra")    
ENDIF(CMAKE_COMPILER_IS_GNUCXX)

add_library(mwalib ${PROGSRC})       # define executable target prog, specify sources
target_include_directories(mwalib PUBLIC ${CMAKE_SOURCE_DIR})
target_link_libraries(mwalib pthread cfitsio)   # -l flags for linking target
