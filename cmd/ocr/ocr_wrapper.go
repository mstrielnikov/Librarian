package main

/*
#cgo LDFLAGS: -L. -locr
#include <stdlib.h>

const char* extract_text(const char* imagePath);
const char* upload_to_elasticsearch(const char* pdfPath, const char* url);
*/
import (
	"C"
	"unsafe"
)

func OCRExtractText(imagePath string) string {
	cImagePath := C.CString(imagePath)
	defer C.free(unsafe.Pointer(cImagePath))

	cText := C.ocr_extract_text(cImagePath)
	return C.GoString(cText)
}
