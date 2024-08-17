import net.sourceforge.tess4j._

object OCRUtil {
  def extractText(imagePath: String): String = {
    val tesseract = new Tesseract()
    try {
      tesseract.doOCR(new java.io.File(imagePath))
    } catch {
      case e: Exception =>
        println(s"Error processing image $imagePath: ${e.getMessage}")
        ""
    }
  }
}