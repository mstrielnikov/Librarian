# Compiler
CXX = g++

# Compiler flags
CXXFLAGS = -Wall -std=c++11 -fPIC

# Libraries
LIBS = -ltesseract -llept

# Output library name
TARGET = libocr.so

# Source files
SRCS = ocr_lib.cpp

# Object files
OBJS = $(SRCS:.cpp=.o)

# Default target
all: $(TARGET)

# Link object files to create the shared library
$(TARGET): $(OBJS)
    $(CXX) $(CXXFLAGS) -shared -o $(TARGET) $(OBJS) $(LIBS)

# Compile source files to object files
%.o: %.cpp
    $(CXX) $(CXXFLAGS) -c $< -o $@

# Clean up build artifacts
clean:
    rm -f $(OBJS) $(TARGET)

# PHONY targets
.PHONY: all clean
