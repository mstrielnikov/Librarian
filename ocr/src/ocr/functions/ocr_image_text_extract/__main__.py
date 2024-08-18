import os
import argparse
from ..lib.minio_client import MinioClient
from ..lib.ocr.ocr_image import OCRUtil
from ..lib.env_util import Environment


def display_help():
    man_page_path = os.path.join(os.path.dirname(__file__), '../resources/ocr_cli_man.txt')
    with open(man_page_path, 'r') as man_file:
        print(man_file.read())


def process_image(image_path):
    text = OCRUtil.extract_text(image_path)
    temp_file_path = f"/tmp/{os.path.basename(image_path)}.txt"
    with open(temp_file_path, 'w') as temp_file:
        temp_file.write(text)
    return temp_file_path


def upload_to_minio(image_path, minio_endpoint, minio_access_key, minio_secret_key, minio_bucket):
    minio_client = MinioClient(
        minio_endpoint=minio_endpoint,
        minio_access_key=minio_access_key,
        minio_secret_key=minio_secret_key,
        minio_bucket=minio_bucket
    )
    temp_file_path = process_image(image_path)
    object_name = f"ocr/{os.path.basename(temp_file_path)}"
    minio_client.upload_file(temp_file_path, object_name)
    os.remove(temp_file_path)


def main():
    # Check if --help flag is present
    if '--help' in os.sys.argv:
        display_help()
        return

    # Load environment variables as defaults
    env = Environment(
        config_file_path="src/main/resources/config.yaml",
        check_env_var=True,
        allow_config_file_nonexistence=True,
        nullable=False
    )

    # Argument parsing
    parser = argparse.ArgumentParser(description="OCR text extraction and upload to MinIO", add_help=False)
    parser.add_argument('--minio-endpoint', default=env.get_env('MINIO_ENDPOINT', 'http://localhost:9000').as_string(), help="MinIO server endpoint")
    parser.add_argument('--minio-access-key', default=env.get_env('MINIO_ACCESS_KEY', 'minioadmin').as_string(), help="MinIO access key")
    parser.add_argument('--minio-secret-key', default=env.get_env('MINIO_SECRET_KEY', 'minioadmin').as_string(), help="MinIO secret key")
    parser.add_argument('--minio-bucket', default=env.get_env('MINIO_BUCKET', 'ocr-results').as_string(), help="MinIO bucket name")
    parser.add_argument('--file-path', required=True, help="Path to the image file for OCR processing")

    args = parser.parse_args()

    # Process the image and upload to MinIO
    upload_to_minio(
        file_path=args.image_path,
        minio_endpoint=args.minio_endpoint,
        minio_access_key=args.minio_access_key,
        minio_secret_key=args.minio_secret_key,
        minio_bucket=args.minio_bucket
    )

if __name__ == "__main__":
    main()
