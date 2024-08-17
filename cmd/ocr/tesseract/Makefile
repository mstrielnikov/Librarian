# Compiler
CXX = g++

# Compiler flags
CXXFLAGS = -Wall -std=c++11

# Libraries
LIBS = -ltesseract -llept -lhpdf -lcurl

# Output binary name
TARGET = ocr_to_pdf

# Source files
SRCS = ocr_to_pdf.cpp

# Object files
OBJS = $(SRCS:.cpp=.o)

# Default target
all: $(TARGET)

# Link object files to create the binary
$(TARGET): $(OBJS)
	$(CXX) $(CXXFLAGS) -o $(TARGET) $(OBJS) $(LIBS)

# Compile source files to object files
%.o: %.cpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Clean up build artifacts
clean:
	rm -f $(OBJS) $(TARGET)

# Run the program
run: $(TARGET)
	./$(TARGET) input_image.png output.pdf http://localhost:9200/pdfs/_doc/1

# PHONY targets
.PHONY: all clean run
