from ocrolib.toplevel import ocropy

class OCRUtil:
    @staticmethod
    def extract_text(image_path):
        try:
            output = ocropy.recognize(image_path)
            return output
        except Exception as e:
            print(f"Error processing image {image_path}: {str(e)}")
