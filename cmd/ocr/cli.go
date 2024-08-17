package main

import (
	"fmt"
	"github.com/elastic/go-elasticsearch/v8"
	"github.com/spf13/cobra"
	"log"
	"os"
)

var Env = client.Environment{
	configFilePath:              ".env",
	configExtension:             ".env",
	allowConfigFileNonExistanse: true,
	checkEnvVar:                 true,
	nullable:                    false,
}

var (
	// Set defaults from environment variables if available
	elasticEndpoint = Env.GetEnv("ELASTIC_ENDPOINT", "").asString()
)

// UploadMinioCmd represents the base command when called without any subcommands
var UploadMinioCmd = &cobra.Command{
	Use:   "ocr <endpoint> <filename>",
	Short: "Command recognise and upload to elasticsearch",
	Long:  `Recognise and upload to elasticsearch by providing elasticsearch endpoint and filename`,
	Run: func(cli *cobra.Command, args []string) {
		if len(args) != 2 {
			log.Fatalf(fmt.Sprintf(`only accepts 2 arguments, got %d`, len(args)))
		}

		endpoint, err := cli.Flags().GetString("endpoint")
		if err != nil {
			log.Fatalf(fmt.Sprintf("endpoint is not defined: %v", err))
		}

		filename, err := cli.Flags().GetString("filename")
		if err != nil {
			log.Fatalf(fmt.Sprintf("error reading filename %s: %v", filename, err))
		}

		pdfData := OCRExtractText(filename)

		// Initialize Elasticsearch client
		es, err := elasticsearch.NewDefaultClient()
		if err != nil {
			log.Fatalf("Error creating Elasticsearch client: %v", err)
		}

		// Send the PDF data to Elasticsearch
		err = ElasticPDFUpload(pdfData, es)
		if err != nil {
			log.Fatalf("Error sending PDF to Elasticsearch: %v", err)
		}

		fmt.Printf("PDF successfully created at %s and sent to Elasticsearch\n", filename)
	},
}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "ocr",
	Short: "OCR tool for PDF files with pages as scans",
	Long:  `OCR service to recognise PDF files with pages as scans and then save as a valid PDF with uploading to elastic search`,
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

	UploadMinioCmd.Flags().StringVar(&elasticEndpoint, "endpoint", minioBucketName, "ElasticSearch endpoint")
	UploadMinioCmd.Flags().String("filename", "", "Filename to upload")
}
