from minio import Minio
from minio.error import S3Error
from env_util import Environment


class MinioClient:
    def __init__(self):
        env = Environment(config_file_path="src/main/resources/config.yaml", check_env_var=True, nullable=False)

        minio_endpoint = env.get_env('MINIO_ENDPOINT', 'http://localhost:9000').as_string()
        access_key = env.get_env('MINIO_ACCESS_KEY', 'minioadmin').as_string()
        secret_key = env.get_env('MINIO_SECRET_KEY', 'minioadmin').as_string()
        bucket_name = env.get_env('MINIO_BUCKET', 'ocr-results').as_string()

        self.client = Minio(
            minio_endpoint,
            access_key=access_key,
            secret_key=secret_key,
            secure=False
        )
        self.bucket_name = bucket_name

        if not self.client.bucket_exists(self.bucket_name):
            self.client.make_bucket(self.bucket_name)

    def upload_file(self, file_path, object_name):
        try:
            self.client.fput_object(
                self.bucket_name, object_name, file_path
            )
            print(f"File uploaded to MinIO: {object_name}")
        except S3Error as e:
            print(f"Failed to upload file: {e}")
