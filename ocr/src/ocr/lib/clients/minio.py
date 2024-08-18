from minio import Minio
from minio.error import S3Error
from .utils.env_util import Environment


class MinioClient:
    def __init__(self, minio_endpoint: str, access_key: str, secret_key: str, bucket_name: str, secure: bool = False):
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
