#include <tesseract/baseapi.h>
#include <leptonica/allheaders.h>
#include <hpdf.h>
#include <curl/curl.h>
#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <stdexcept>
#include <sstream>

// Function to extract text from an image using Tesseract
std::string extractTextFromImage(const std::string& imagePath) {
    tesseract::TessBaseAPI ocr;
    if (ocr.Init(NULL, "eng")) {
        throw std::runtime_error("Could not initialize tesseract.");
    }

    Pix *image = pixRead(imagePath.c_str());
    ocr.SetImage(image);

    std::string outText = ocr.GetUTF8Text();
    ocr.End();
    pixDestroy(&image);

    return outText;
}

// Function to create a PDF file with extracted text using libharu
void createPDF(const std::string& text, const std::string& outputPath) {
    HPDF_Doc pdf = HPDF_New(NULL, NULL);
    if (!pdf) {
        throw std::runtime_error("Could not create PDF object.");
    }

    HPDF_Page page = HPDF_AddPage(pdf);
    HPDF_Page_SetSize(page, HPDF_PAGE_SIZE_A4, HPDF_PAGE_PORTRAIT);

    HPDF_Page_SetFontAndSize(page, HPDF_GetFont(pdf, "Helvetica", NULL), 12);

    HPDF_Page_BeginText(page);
    HPDF_Page_MoveTextPos(page, 50, HPDF_Page_GetHeight(page) - 50);

    std::istringstream stream(text);
    std::string line;
    while (std::getline(stream, line)) {
        HPDF_Page_ShowTextNextLine(page, line.c_str());
    }

    HPDF_Page_EndText(page);

    HPDF_SaveToFile(pdf, outputPath.c_str());
    HPDF_Free(pdf);
}

// Function to send PDF data to Elasticsearch using libcurl
void sendPDFToElasticsearch(const std::string& pdfPath, const std::string& url) {
    std::ifstream file(pdfPath, std::ios::binary);
    std::ostringstream oss;
    oss << file.rdbuf();
    std::string pdfData = oss.str();

    std::string base64Data = "data:application/pdf;base64," + std::string(base64_encode(pdfData));

    CURL *curl;
    CURLcode res;

    curl_global_init(CURL_GLOBAL_DEFAULT);
    curl = curl_easy_init();

    if(curl) {
        struct curl_slist *headers = NULL;
        headers = curl_slist_append(headers, "Content-Type: application/json");

        std::string jsonBody = "{\"pdf_data\":\"" + base64Data + "\"}";

        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_POST, 1L);
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, jsonBody.c_str());

        res = curl_easy_perform(curl);
        if(res != CURLE_OK) {
            throw std::runtime_error("Failed to send data to Elasticsearch: " + std::string(curl_easy_strerror(res)));
        }

        curl_easy_cleanup(curl);
    }

    curl_global_cleanup();
}

int main(int argc, char* argv[]) {
    if (argc < 4) {
        std::cerr << "Usage: " << argv[0] << " <input_image_path> <output_pdf_path> <elasticsearch_url>" << std::endl;
        return 1;
    }

    try {
        std::string inputImagePath = argv[1];
        std::string outputPDFPath = argv[2];
        std::string elasticsearchURL = argv[3];

        // Extract text from image
        std::string text = extractTextFromImage(inputImagePath);

        // Create PDF with the extracted text
        createPDF(text, outputPDFPath);

        // Send PDF to Elasticsearch
        sendPDFToElasticsearch(outputPDFPath, elasticsearchURL);

        std::cout << "PDF successfully created at " << outputPDFPath << " and sent to Elasticsearch" << std::endl;

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}
