package minio

import (
	"context"
	"fmt"
	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
	"log"
	"path/filepath"
)

type MinioClient struct {
	MinioClient *minio.Client
	minioConfig *MinioConfig
}

// NewMinioClient to initialize MinIO client
func NewMinioClient(minioConfig *MinioConfig) *MinioClient {
	minioClient, err := minio.New(minioConfig.endpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(minioConfig.accessKeyID, minioConfig.secretAccessKey, ""),
		Secure: minioConfig.useSSL,
	})
	if err != nil {
		log.Fatalln(err)
	}

	return &MinioClient{
		MinioClient: minioClient,
		minioConfig: minioConfig,
	}
}

// CreateBucket to create bucket
func (m *MinioClient) CreateBucket(bucketName string) {
	err := m.MinioClient.MakeBucket(context.Background(), bucketName, minio.MakeBucketOptions{})
	if err != nil {
		log.Fatalln(err)
	}

	fmt.Printf("Successfully created bucket %s\n", bucketName)
}

// UploadFile to upload the file into minio
func (m *MinioClient) UploadFile(bucketName, filePath string) {
	fileName := filepath.Base(filePath)
	_, err := m.MinioClient.FPutObject(context.Background(), bucketName, fileName, filePath, minio.PutObjectOptions{})
	if err != nil {
		log.Fatalln(err)
	}

	fmt.Printf("Successfully uploaded %s to %s/%s\n", filePath, bucketName, fileName)
}
