package main

import (
	"bytes"
	"encoding/base64"
	"fmt"
)

func ElasticUploadPDF(pdfData []byte, es *elasticsearch.Client) error {
	// Encode PDF to base64
	pdfBase64 := base64.StdEncoding.EncodeToString(pdfData)

	// Define the Elasticsearch document
	doc := map[string]interface{}{
		"pdf_data": pdfBase64,
	}

	// Send the document to Elasticsearch
	req := esapi.IndexRequest{
		Index:      "pdfs", // Elasticsearch index
		DocumentID: "1",    // Unique ID for the document
		Body:       bytes.NewReader([]byte(fmt.Sprintf(`{"pdf_data":"%s"}`, pdfBase64))),
		Refresh:    "true",
	}

	res, err := req.Do(context.Background(), es)
	if err != nil {
		return fmt.Errorf("failed to index document: %w", err)
	}
	defer res.Body.Close()

	if res.IsError() {
		return fmt.Errorf("error indexing document: %s", res.String())
	}

	return nil
}
