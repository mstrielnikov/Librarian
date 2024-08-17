package main

import (
	"bytes"
	"fmt"
)

// Create PDF from text
func PDFFromString(text string) ([]byte, error) {
	// Create a new PDF
	pdf := gofpdf.New("P", "mm", "A4", "")

	// Add a page to the PDF
	pdf.AddPage()

	// Set font for the PDF
	pdf.SetFont("Arial", "", 12)

	// Add the text to the PDF
	pdf.MultiCell(0, 10, text, "", "", false)

	// Save PDF to a buffer instead of a file
	var buf bytes.Buffer
	err := pdf.Output(&buf)
	if err != nil {
		return nil, fmt.Errorf("failed to create PDF: %w", err)
	}

	return buf.Bytes(), nil
}
