package main

import (
	"fmt"
	"github.com/spf13/cobra"
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
	minioEndpoint        = Env.GetEnv("MINIO_ENDPOINT", "").asString()
	minioAccessKeyID     = Env.GetEnv("MINIO_ACCESS_KEY", "").asString()
	minioSecretAccessKey = Env.GetEnv("MINIO_SECRET_KEY", "").asString()
	minioBucketName      = Env.GetEnv("MINIO_BUCKET", "").asString()
	minioUseSSL          = Env.GetEnv("MINIO_USE_SSL", "").asBool()
)

// UploadMinioCmd represents the base command when called without any subcommands
var UploadMinioCmd = &cobra.Command{
	Use:   "client <bucket> <key> <file>",
	Short: "Command to upload documents to minio",
	Long:  `Upload documents to minio by providing file name to a bucket`,
	Run: func(cli *cobra.Command, args []string) {
		if len(args) != 6 {
			log.Fatalf(fmt.Sprintf(`only accepts 5 arguments, got %d`, len(args)))
		}

		bucket, err := cli.Flags().GetString("bucket")
		if err != nil {
			log.Fatalf(fmt.Sprintf("bucket is not defined: %v", err))
		}

		filename, err := cli.Flags().GetString("filename")
		if err != nil {
			log.Fatalf(fmt.Sprintf("error reading filename %s: %v", filename, err))
		}

		minioClient := NewMinioClient(minioEndpoint, minioAccessKeyID, minioSecretAccessKey, minioUseSSL)
		minioClient.UploadFile(bucket, filename)
	},
}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "client",
	Short: "User client to upload documents",
	Long:  `User CLI client to upload documents in minio storage or any S3 compatible one for further processing`,
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

	UploadMinioCmd.Flags().StringVar(&minioEndpoint, "endpoint", minioEndpoint, "MinIO endpoint")
	UploadMinioCmd.Flags().StringVar(&minioAccessKeyID, "accessKey", minioAccessKeyID, "Access key ID")
	UploadMinioCmd.Flags().StringVar(&minioSecretAccessKey, "secretKey", minioSecretAccessKey, "Secret access key")
	UploadMinioCmd.Flags().BoolVar(&minioUseSSL, "ssl", minioUseSSL, "Use SSL")
	UploadMinioCmd.Flags().StringVar(&minioBucketName, "bucket", minioBucketName, "Bucket name")
	UploadMinioCmd.Flags().String("filename", "", "Filename to upload")
}
