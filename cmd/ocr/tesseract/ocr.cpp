// ocr_lib.cpp
#include <tesseract/baseapi.h>
#include <leptonica/allheaders.h>
#include <string>

extern "C" {
    const char* ocr_extract_text(const char* imagePath) {
        static std::string outText;

        tesseract::TessBaseAPI ocr;
        if (ocr.Init(NULL, "eng")) {
            return "Could not initialize tesseract.";
        }

        Pix *image = pixRead(imagePath);
        ocr.SetImage(image);

        outText = ocr.GetUTF8Text();
        ocr.End();
        pixDestroy(&image);

        return outText.c_str();
    }
}
