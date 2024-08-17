package main

import (
	"fmt"
	"io/ioutil"
)

func OCRExtractTextFromPDF(pdfPath string) (string, error) {
	// Read the file
	fileBytes, err := ioutil.ReadFile(pdfPath)
	if err != nil {
		return "", fmt.Errorf("failed to read PDF file: %w", err)
	}

	// Create a new OCR client
	client := gosseract.NewClient()
	defer client.Close()

	// Set the image from the file bytes
	err = client.SetImageFromBytes(fileBytes)
	if err != nil {
		return "", fmt.Errorf("failed to set image: %w", err)
	}

	// Extract text from the image
	text, err := client.Text()
	if err != nil {
		return "", fmt.Errorf("failed to extract text: %w", err)
	}

	return text, nil
}
