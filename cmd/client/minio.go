package main

import (
	"context"
	"fmt"
	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
	"log"
	"os"
	"path/filepath"
)

type MinioClient struct {
	minioClient *minio.Client
}

// NewMinioClient to initialize MinIO client
func NewMinioClient(minioEndpoint, minioAccessKeyID, minioSecretAccessKey string, minioUseSSL bool) *MinioClient {
	minioClient, err := minio.New(minioEndpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(minioAccessKeyID, minioSecretAccessKey, ""),
		Secure: minioUseSSL,
	})
	if err != nil {
		log.Fatalln(err)
	}

	return &MinioClient{
		minioClient: minioClient,
	}
}

// CreateBucket to create bucket
func (m *MinioClient) CreateBucket(bucketName string) {
	err := m.minioClient.MakeBucket(context.Background(), bucketName, minio.MakeBucketOptions{})
	if err != nil {
		log.Fatalln(err)
	}

	fmt.Printf("Successfully created bucket %s\n", bucketName)
}

// UploadFile to upload the file into minio
func (m *MinioClient) UploadFile(bucketName, filePath string) {
	fileName := filepath.Base(filePath)

	if !fileExists(fileName) {
		log.Fatalf(fmt.Sprintf("File %s does not exist", fileName))
	}

	_, err := m.minioClient.FPutObject(context.Background(), bucketName, fileName, filePath, minio.PutObjectOptions{})
	if err != nil {
		log.Fatalln(err)
	}

	fmt.Printf("Successfully uploaded %s to %s/%s\n", filePath, bucketName, fileName)
}

// fileExists checks if the specified file exists.
func fileExists(filePath string) bool {
	_, err := os.Stat(filePath)
	if os.IsNotExist(err) {
		return false
	}
	return err == nil
}
