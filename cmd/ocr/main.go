package main

/*
#cgo LDFLAGS: -L. -locr
#include <stdlib.h>

const char* extract_text(const char* imagePath);
const char* upload_to_elasticsearch(const char* pdfPath, const char* url);
*/

func main() {
	Execute()
}
