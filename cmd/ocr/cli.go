package main

import (
	"fmt"
	"github.com/spf13/cobra"
	"io/ioutil"
	"log"
	"os"
)

var Env = Environment{
	configFilePath:              ".env",
	configExtension:             ".env",
	allowConfigFileNonExistanse: true,
	checkEnvVar:                 true,
	nullable:                    false,
}

var (
	// Set defaults from environment variables if available
	elasticEndpoint = Env.GetEnv("ELASTICSEARCH_ENDPOINT", "").asString()
)

// UploadMinioCmd represents the base command when called without any subcommands
var UploadMinioCmd = &cobra.Command{
	Use:   "ocr <filename> <elasticserarch_endpoint>",
	Short: "Recognise and upload PDF to Elasticsearch",
	Long:  `Upload OCR recognsed documents to elasticsearch by providing filename and elasticsearch endpoint.`,
	Run: func(cli *cobra.Command, args []string) {
		if len(args) != 2 {
			log.Fatalf(fmt.Sprintf(`only accepts 2 arguments, got %d`, len(args)))
		}

		filename, err := cli.Flags().GetString("filename")
		if err != nil {
			log.Fatalf(fmt.Sprintf("filename is not defined: %v", err))
		}

		endpoint, err := cli.Flags().GetString("endpoint")
		if err != nil {
			log.Fatalf(fmt.Sprintf("filename is not defined: %v", err))
		}

		text, err := OCRExtractTextFromPDF(filename)
		if err != nil {
			log.Fatalf("Error extracting text from PDF: %v", err)
		}

		// Create a new PDF with the extracted text
		pdfData, err := PDFFromString(text)
		if err != nil {
			log.Fatalf("Error creating PDF: %v", err)
		}

		// Save the PDF to disk
		err = ioutil.WriteFile(outputPDFPath, pdfData, 0644)
		if err != nil {
			log.Fatalf("Error saving PDF to disk: %v", err)
		}

		// Initialize Elasticsearch client
		es, err := elasticsearch.NewDefaultClient()
		if err != nil {
			log.Fatalf("Error creating Elasticsearch client: %v", err)
		}

		// Send the PDF data to Elasticsearch
		err = ElasticUploadPDF(pdfData, es)
		if err != nil {
			log.Fatalf("Error sending PDF to Elasticsearch: %v", err)
		}

		fmt.Printf("PDF successfully created at %s and sent to Elasticsearch\n", outputPDFPath)

	},
}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "ocr",
	Short: "Optical Character Recognition service",
	Long: `Optical Character Recognition service for recognition of PDF docs with pages presented as scans.
This tool recognises this docs and then creates PDF with corrected formatted pages and extractable text. Recognised data upload to elasticsearch`,
	// Run: func(cli *cobra.Command, args []string) { },
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}

	err = UploadMinioCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	// rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.client.yaml)")

	// Cobra also supports local flags, which will only run
	// when this action is called directly.
	rootCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
	rootCmd.AddCommand(UploadMinioCmd)

	UploadMinioCmd.Flags().String("filename", "", "Filename to analyze")
	UploadMinioCmd.Flags().StringVar(&elasticEndpoint, "endpoint", elasticEndpoint, "ElasticSearch endpoint")
}
